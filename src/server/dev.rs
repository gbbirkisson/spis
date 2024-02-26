use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, _msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {}
}

async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWs {}, &req, stream)
}

pub fn create_socket() -> actix_web::Route {
    web::get().to(ws)
}
