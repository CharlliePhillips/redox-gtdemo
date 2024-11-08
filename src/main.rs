use libredox::call::write;
use log::{info, warn, LevelFilter};
use redox_log::{OutputBuilder, RedoxLogger};

use redox_scheme::{RequestKind, SchemeMut, SignalBehavior, Socket, V2};
use std::{borrow::BorrowMut, fs::{File, OpenOptions}, io::{Read, Write}, os::{fd::AsRawFd, unix::fs::OpenOptionsExt}};

use scheme::{GTDemoScheme};

mod scheme;

enum Ty {
    GTDemo,
}

fn main() {
    let _ = RedoxLogger::new()
    .with_output(
        OutputBuilder::stdout()
            .with_filter(log::LevelFilter::Debug)
            .with_ansi_escape_codes()
            .build()
    )
    .with_process_name("gtdemo".into())
    .enable();
    info!("gtdemo logger started");
    
    //get arg 0 (name used to start)
    let ty = match &*std::env::args().next().unwrap() {
        "gtdemo" => Ty::GTDemo,
        _ => panic!("needs to be called as gtdemo"),
    };

    redox_daemon::Daemon::new(move |daemon| {
        let name = match ty {
            Ty::GTDemo => "gtdemo",
        };
        let socket = Socket::<V2>::create(name).expect("gtdemo: failed to create demo scheme");
        let mut demo_scheme= GTDemoScheme(ty, 1);

        libredox::call::setrens(0, 0).expect("gtdemo: failed to enter null namespace");

        daemon.ready().expect("gtdemo: failed to notify parent");

        loop {
            info!("gtdemo daemon loop start");
            // dd if=/scheme/gtdemo of=/scheme/null count=1 
            let Some(request) = socket
                .next_request(SignalBehavior::Restart)
                .expect("gtdemo: failed to read events from demo scheme")
            else {
                warn!("exiting gtdemo");
                std::process::exit(0);
            };

            match request.kind() {
                RequestKind::Call(request) => {
                    //this buffer is ignored, we care about the usize value in GTDemoScheme struct
                    let readbuf: &mut [u8] = &mut [];
                    let scheme_before = demo_scheme.read(0, readbuf, 0, 0)
                        .expect("failed to read before request/response");
                    info!("scheme read before request/response: {scheme_before}");

                    let response = request.handle_scheme_mut(&mut demo_scheme);

                    socket
                        .write_responses(&[response], SignalBehavior::Restart)
                        .expect("gtdemo: failed to write responses to demo scheme");

                    let response_data = demo_scheme.read(0, readbuf, 0, 0)
                        .expect("failed to read after request/response");
                    info!("scheme read after response: {response_data:?}");
                    
                    
                    //let mut buzzd = OpenOptions::new()
                    //    .create(true)
                    //    .read(true)
                    //    .write(true)
                    //    .open("/scheme/buzz")
                    //    .expect("failed to open buzz scheme");
                    //let gtdemo_str: &mut [u8] = &mut [b'G', b'T', b' ', b'D', b'E', b'M', b'O'];
                    //buzzd.write(gtdemo_str).expect("failed to write gtdemo to buzz");
                }
                _ => (),
            }
            //info!("running gtdemo daemon")
        }
    })
    .expect("gtdemo: failed to daemonize");
}