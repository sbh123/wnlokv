extern crate futures;
extern crate grpcio;
extern crate protos;

use std::io::Read;
use std::sync::Arc;
use std::{io, thread};
use std::time::Duration;

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::raftpb::HelloRequest;
use protos::raftpb::HelloReply;
use protos::raftpb_grpc;
use protos::raftpb_grpc::Greeter;

#[derive(Clone)]
struct GreeterService;

impl Greeter for GreeterService {
    fn say_hello(&mut self, ctx: RpcContext, req: HelloRequest,
                 sink: UnarySink<HelloReply>) {
 
        println!("Recive a req: {:?}", req);
        let msg = format!("Hello {}", req.get_name());
        thread::sleep(Duration::from_secs(1));
        let mut resp = HelloReply::new();
        resp.set_message(msg);
        let f = sink.success(resp)
                .map_err(move |e| eprintln!("Fail to reply {:?}: {:?}", req, e));
        ctx.spawn(f);
    }
}



fn main() {
    let env = Arc::new(Environment::new(1));
    let service = raftpb_grpc::create_greeter(GreeterService);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 50051)
        .build().unwrap();

    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("Listening on {}:{}", host, port);
    }

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press Enter to exit ...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(());
    });

    let _ = rx.wait();
    let _ = server.shutdown().wait();
}
