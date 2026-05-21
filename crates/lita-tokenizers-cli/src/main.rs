mod main_impl;
use main_impl::Error;
use main_impl::main_impl;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let fut = main_impl();
    let ctrl_c = tokio::signal::ctrl_c();

    tokio::select! {
        res = fut => res,
        _ = ctrl_c => Ok(()),
    }
}
