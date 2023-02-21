

pub async fn get<T>(context: worker::RouteContext<T>) -> worker::Result<worker::Response> {
    return Ok(worker::Response::ok("Hell, sub_dir/other/mod.rs")?);
}

