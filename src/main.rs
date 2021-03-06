use std::ffi::OsStr;
use std::io::Write;
use std::panic;
use std::process::{Command, Stdio};
use std::time::Instant;

#[cfg(any(feature = "png", feature = "jpeg"))]
use imagefmt::ColFmt;

#[cfg(any(feature = "png", feature = "jpeg"))]
use itertools::{iproduct, Itertools};

use structopt::clap::Format;
use structopt::StructOpt;

mod cli;
use cli::Cli;
mod ffi;
mod screenshot;
use screenshot::Screenshot;
mod macros;
use macros::*;

fn main() {
    // parse args, handle custom `--version`
    let args = Cli::from_args();
    if args.version {
        eprintln!(
            "{} v{} compiled for '{}' at {} ({}@{})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("TARGET"),
            env!("TIME"),
            env!("GIT_BRANCH"),
            env!("GIT_COMMIT")
        );
        return;
    }
    unsafe { DEBUG = args.debug };
    debug!("Found args: {:#?}", args);

    // take the screenshot
    timer_start!(screenshot);
    let shot =
        Screenshot::capture().unwrap_or_else(|e| color_panic!("Failed to take screenshot: {}", e));
    timer_time!("Capturing screenshot", screenshot);
    debug!("Found monitors: {:?}", shot.monitors);

    // blur
    if let Some(r) = args.radius {
        timer_start!(blur);
        ffi::blur(
            shot.data,
            shot.width() as libc::c_int,
            shot.height() as libc::c_int,
            r as libc::c_int,
        );
        timer_time!("Blurring", blur);
    }

    // overlay/invert on each monitor
    #[cfg(any(feature = "png", feature = "jpeg"))]
    {
        if let Some(path) = args.path {
            timer_start!(decode);
            let image = imagefmt::read(path, ColFmt::BGRA)
                .unwrap_or_else(|e| color_panic!("Failed to read image: {}", e));
            timer_time!("Decoding image", decode);

            for (i, (w, h)) in shot
                .monitors
                .iter()
                .cloned()
                .map(|(a, b)| (a as usize, b as usize)) // map_into doesn't work with tuples
                .enumerate()
            {
                if args.ignore.contains(&i) {
                    debug!("Ignoring monitor {}", i);
                    continue;
                }

                if image.w > w || image.h > h {
                    eprintln!(
                        "{}",
                        Format::Warning(
                            "Your image is larger than your monitor, image positions may be off!"
                        )
                    );
                }

                let (mut x_off, mut y_off) = if args.pos.is_empty() {
                    (
                        ((w / 2).saturating_sub(image.w / 2)) as isize,
                        ((h / 2).saturating_sub(image.h / 2)) as isize,
                    )
                } else {
                    args.pos.iter().cloned().collect_tuple().unwrap() // exactly two items validated by clap/structopt
                };

                while x_off.is_negative() {
                    x_off += w as isize;
                }
                while y_off.is_negative() {
                    y_off += h as isize;
                }
                while x_off >= shot.width() as isize {
                    x_off -= w as isize;
                }
                while y_off >= shot.height() as isize {
                    y_off -= h as isize;
                }

                let (x_off, y_off) = (x_off as usize, y_off as usize);
                debug!(
                    "Calculated image position on monitor {}: ({},{})",
                    i, x_off, y_off
                );

                // should be able to rewrite this to write rows at once
                timer_start!(overlay);
                for (x, y) in iproduct!(0..image.w, 0..image.h) {
                    let i_dst = (x + x_off + w * (y + y_off)) * 4;
                    let i_src = (x + image.w * y) * 4;

                    let src_bgra = unsafe { image.buf.get_unchecked(i_src..i_src + 4) };
                    let (src_bgr, src_a) = src_bgra.split_at(3);
                    let src_a = unsafe { src_a.get_unchecked(0) };

                    // skip invisible pixels
                    if *src_a == 0 {
                        continue;
                    }

                    // dst_a not used
                    if let Some(dst_bgr) = shot.data.get_mut(i_dst..i_dst + 3) {
                        if args.invert {
                            dst_bgr.iter_mut().for_each(|c| *c = !*c)
                        } else if *src_a == 255 {
                            dst_bgr.copy_from_slice(src_bgr) // opaque pixels are a dumb copy
                        } else {
                            // anything else needs alpha blending
                            let a = *src_a as usize + 1;
                            let inv_a = 257 - a;
                            dst_bgr
                                .iter_mut()
                                .zip(src_bgr.iter())
                                .for_each(|(dst_c, &src_c)| {
                                    *dst_c =
                                        ((a * *dst_c as usize + inv_a * src_c as usize) >> 8) as u8
                                });
                        }
                    }
                }
                timer_time!("Overlaying/inverting image", overlay);
            }
        }
    }

    //TODO draw text

    // call i3lock and pass image bytes
    // this is a bit gross
    let nofork = args.i3lock.contains(&OsStr::new("-n").to_os_string())
        || args.i3lock.contains(&OsStr::new("--nofork").to_os_string());

    debug!("Calling i3lock with args: {:?}", args.i3lock);
    let mut cmd = Command::new("i3lock")
        .args(&[
            "-i",
            "/dev/stdin",
            &format!("--raw={}x{}:bgrx", shot.width(), shot.height()),
        ])
        .args(args.i3lock)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| color_panic!("Failed to call i3lock: {}", e));

    cmd.stdin
        .as_mut()
        .unwrap_or_else(|| color_panic!("Failed to open i3lock stdin!"))
        .write_all(shot.data)
        .unwrap_or_else(|e| color_panic!("Failed to write image to i3lock stdin: {}", e));

    if nofork {
        debug!("Asked i3lock not to fork, calling wait()");
        let _ = cmd.wait();
    }
}
