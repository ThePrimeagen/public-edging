pub mod sub_dir;
mod other_file;

pub async fn get<T>(context: worker::RouteContext<T>) -> worker::Result<worker::Response> {
    return Ok(worker::Response::ok("Hell, from lib.rs")?);
}


