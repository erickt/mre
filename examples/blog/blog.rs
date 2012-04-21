import app::app;

fn main() {
    let app = app();
    routes::routes(app);
    app.run();
}
