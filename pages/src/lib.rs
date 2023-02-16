pub mod sub_dir;
mod other_file;

pub fn get(context: worker::Context) -> worker::Result {
    Ok(worker::Response::new("Hello, world!"))
}


