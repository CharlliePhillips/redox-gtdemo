use log::{info, warn, LevelFilter};
use redox_log::{OutputBuilder, RedoxLogger};

use redox_scheme::{RequestKind, SchemeMut, SignalBehavior, Socket, V2};

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
        let mut demo_scheme= GTDemoScheme(ty);

        libredox::call::setrens(0, 0).expect("gtdemo: failed to enter null namespace");

        daemon.ready().expect("gtdemo: failed to notify parent");

        loop {
            info!("gtdemo daemon loop start");

            let Some(request) = socket
                .next_request(SignalBehavior::Restart)
                .expect("gtdemo: failed to read events from demo scheme")
            else {
                warn!("exiting gtdemo");
                std::process::exit(0);
            };
            info!("request: {request:?}");

            match request.kind() {
                RequestKind::Call(request) => {
                    let response = request.handle_scheme_mut(&mut demo_scheme);

                    socket
                        .write_responses(&[response], SignalBehavior::Restart)
                        .expect("gtdemo: failed to write responses to demo scheme");
                }
                _ => (),
            }
            info!("running gtdemo daemon")
        }
    })
    .expect("gtdemo: failed to daemonize");
}