#[macro_use]
extern crate log;
extern crate env_logger;

extern crate jsonrpc_ipc_server;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_core;

use std::sync::Arc;
use jsonrpc_ipc_server::{ServerBuilder, MetaExtractor, RequestContext};
use jsonrpc_ipc_server::jsonrpc_core::*;
use jsonrpc_pubsub::{Session, PubSubMetadata};

use std::time::Instant;
// use jsonrpc_core::*;
use jsonrpc_core::futures::Future;


// 
// 
// HERE IS THE MEAT
// 

#[derive(Debug, Clone)]
/// RPC methods metadata.
pub struct Metadata {
	/// Request PubSub Session
	pub session: Arc<Session>
	//  ^^^^^^^^^^^^^^^^^^^^^ --- Holding a PubSubSession
	//                            internally holds reference to _sender_
}

pub struct RpcExtractor;

impl MetaExtractor<Metadata> for RpcExtractor {
	fn extract(&self, req: &RequestContext) -> Metadata {
		trace!(target:"ipc", "ext");
		Metadata {
			session: Arc::new(Session::new(req.sender.clone()))
			//                             ^^^^^^^^^^^^^^^^^^ --- holding reference to sender
		}
	}
}





// 
// 
//  THIS IS BOILERPLATE. IGNORE 
//
// 



impl jsonrpc_core::Metadata for Metadata {}
impl PubSubMetadata for Metadata {
	fn session(&self) -> Option<Arc<Session>> {
		Some(self.session.clone())
	}
}

#[derive(Default)]
struct MyMiddleware {}
impl Middleware<Metadata> for MyMiddleware {
	type Future = FutureResponse;

	fn on_request<F, X>(&self, request: Request, meta: Metadata, next: F) -> FutureResponse where
		F: FnOnce(Request, Metadata) -> X + Send,
		X: Future<Item=Option<Response>, Error=()> + Send + 'static,
	{
		let start = Instant::now();

		Box::new(next(request, meta).map(move |res| {
			println!("Processing took: {:?}", start.elapsed());
			res
		}))
	}
}




fn main() {

    env_logger::init();
	let mut io = MetaIoHandler::with_middleware(MyMiddleware::default());
	io.add_method("say_hello", |_params| {
		Ok(Value::String("hello".into()))
	});

	ServerBuilder::with_meta_extractor(io, RpcExtractor)
				.start("/tmp/json-ipc-test.ipc")
				.expect("Couldn't open socket")
				.wait()
}