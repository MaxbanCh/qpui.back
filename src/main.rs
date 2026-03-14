use actix_web::{rt, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, post};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::mpsc;
use actix_cors::Cors;

type ClientId = usize;
type ClientTx = mpsc::UnboundedSender<String>;

struct AppState {
    rooms: Mutex<HashMap<String, HashMap<ClientId, ClientTx>>>,
    next_client_id: AtomicUsize,
}

impl AppState {
    fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
            next_client_id: AtomicUsize::new(1),
        }
    }

    fn create_room(&self, room_id: &str) -> bool {
        let mut rooms = self.rooms.lock().unwrap();

        if rooms.contains_key(room_id) {
            return false;
        }

        rooms.insert(room_id.to_string(), HashMap::new());
        true
    }

    fn room_exists(&self, room_id: &str) -> bool {
        let rooms = self.rooms.lock().unwrap();
        rooms.contains_key(room_id)
    }

    fn join_room(&self, room_id: &str) -> Option<(ClientId, mpsc::UnboundedReceiver<String>)> {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.get_mut(room_id)?;

        let client_id = self.next_client_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::unbounded_channel();

        room.insert(client_id, tx);

        Some((client_id, rx))
    }

    fn leave_room(&self, room_id: &str, client_id: ClientId) {
        let mut rooms = self.rooms.lock().unwrap();

        if let Some(room) = rooms.get_mut(room_id) {
            room.remove(&client_id);

            if room.is_empty() {
                rooms.remove(room_id);
            }
        }
    }

    fn broadcast(&self, room_id: &str, message: String) -> usize {
        let clients = {
            let rooms = self.rooms.lock().unwrap();

            match rooms.get(room_id) {
                Some(room) => room.values().cloned().collect::<Vec<_>>(),
                None => return 0,
            }
        };

        for tx in &clients {
            let _ = tx.send(message.clone());
        }

        clients.len()
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/rooms/{room_id}")]
async fn create_room(room_id: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    if state.create_room(&room_id) {
        HttpResponse::Created().body("room created")
    } else {
        HttpResponse::Conflict().body("room already exists")
    }
}

#[post("/rooms/{room_id}/broadcast")]
async fn broadcast_to_room(
    room_id: web::Path<String>,
    body: String,
    state: web::Data<AppState>,
) -> impl Responder {
    let sent = state.broadcast(&room_id, body);

    if sent == 0 {
        HttpResponse::NotFound().body("room not found or empty")
    } else {
        HttpResponse::Ok().body(format!("sent to {sent} client(s)"))
    }
}

async fn room_ws(
    room_id: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let room_id = room_id.into_inner();

    if !state.room_exists(&room_id) {
        return Ok(HttpResponse::NotFound().body("room not found"));
    }

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let Some((client_id, mut client_rx)) = state.join_room(&room_id) else {
        return Ok(HttpResponse::NotFound().body("room not found"));
    };

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let state = state.clone();

    rt::spawn(async move {
        loop {
            tokio::select! {
                ws_msg = stream.next() => {
                    match ws_msg {
                        Some(Ok(AggregatedMessage::Text(text))) => {
                            state.broadcast(&room_id, text.to_string());
                        }
                        Some(Ok(AggregatedMessage::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(AggregatedMessage::Close(_))) => {
                            break;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(_)) | None => {
                            break;
                        }
                    }
                }

                server_msg = client_rx.recv() => {
                    match server_msg {
                        Some(server_msg) => {
                            if session.text(server_msg).await.is_err() {
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }

        state.leave_room(&room_id, client_id);
    });

    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4200")
            .allowed_origin("http://127.0.0.1:4200")
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE, actix_web::http::header::ACCEPT])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(hello)
            .service(echo)
            .service(create_room)
            .service(broadcast_to_room)
            .route("/hey", web::get().to(manual_hello))
            .route("/ws/{room_id}", web::get().to(room_ws))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}