use worker::*;

/*
pub fn add_routes<D>(router: Router<D>) -> Router<D> {

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    //
    return router
        //.get_async("/", |_, ctx| async move { })
        .get_async("/", move |ctx| {
            return pages::get(ctx);
        })
        .get_async("/sub_dir/other/", move |ctx| {
            return pages::sub_dir::other::get(ctx);
        })
}
*/

