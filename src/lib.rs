#[macro_use]
extern crate tower_web;
use hyper::Response;
use web3::{
    api::Web3,
    futures::future::{ok, Future},
    transports::Http,
    types::Address,
};

impl_web! {
    #[derive(Clone)]
    pub struct MyStruct {
        pub endpoint: String,
        pub address: Address,
    }

    impl MyStruct {

        #[get("/")]
        fn foo(&self) -> impl Future<Item = Response<String>, Error = Response<String>> {
            self.bar().map_err(|_err| {
                Response::builder().body("SomeError".to_string()).unwrap()
            })
            .and_then(move |_| {
                ok(Response::builder().body("SomeResponse".to_string()).unwrap())
            })
        }

        #[get("/1")]
        fn foo_wait(&self) -> impl Future<Item = Response<String>, Error = Response<String>> {
            self.bar_wait().map_err(|_err| {
                Response::builder().body("SomeError".to_string()).unwrap()
            })
            .and_then(move |_| {
                ok(Response::builder().body("SomeResponse".to_string()).unwrap())
            })
        }

        fn bar(&self) -> Box<dyn Future<Item = (), Error = ()> + Send> {
            let (_eloop, transport) = Http::new(&self.endpoint).unwrap();
            let web3 = Web3::new(transport);
            Box::new(
                web3.eth()
                .transaction_count(self.address.clone(), None)
                .map_err(move |err| println!("Couldn't fetch nonce. Got error: {:#?}", err))
                .and_then(move |nonce| {
                    println!("Obtained nonce {}", nonce);
                    Ok(())
                })
            )
        }

        fn bar_wait(&self) -> Box<dyn Future<Item = (), Error = ()> + Send> {
            let (_eloop, transport) = Http::new(&self.endpoint).unwrap();
            let web3 = Web3::new(transport);
            let _nonce = web3.eth().transaction_count(self.address.clone(), None).wait().unwrap();
            Box::new(ok(()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tokio;

    #[test]
    fn future_fails() {
        let s = MyStruct {
            address: Address::from_str("3cdb3d9e1b74692bb1e3bb5fc81938151ca64b02").unwrap(),
            endpoint: "http://127.0.0.1:8545".to_string(),
        };

        // this fails with the timeout error :/
        // ganache-cli endpoint does not even get hit.
        s.foo().wait().unwrap_err();

        // fails also inside a runtime :/
        // ganache-cli endpoint does not even get hit.
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(s.foo()).unwrap_err();
    }

    #[test]
    fn wait_works() {
        let s = MyStruct {
            address: Address::from_str("3cdb3d9e1b74692bb1e3bb5fc81938151ca64b02").unwrap(),
            endpoint: "http://127.0.0.1:8545".to_string(),
        };

        // this works!
        s.foo_wait().wait().unwrap();

        // this also works!
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(s.foo_wait()).unwrap();
    }
}
