use gzlib::proto::latex::{latex_server::*, Content, Pdf};
use latex_microservice::{prelude::*, processer::LatexObject};
use std::env;
use std::error::Error;
use tokio::sync::oneshot;
use tonic::{transport::Server, Request, Response, Status};

use gzlib::proto;

mod prelude;

struct LatexService;

impl LatexService {
  async fn process(&self, r: Content) -> ServiceResult<Vec<u8>> {
    // Create latex object
    let mut latex_object = LatexObject::new(r.main_latex_file, r.attachments);
    let res = latex_object
      .to_pdf()
      .await
      .map_err(|e| ServiceError::bad_request(&e))?;
    Ok(res)
  }
}

#[tonic::async_trait]
impl Latex for LatexService {
  async fn process(
    &self,
    request: Request<Content>,
  ) -> Result<Response<proto::latex::Pdf>, Status> {
    let res = self.process(request.into_inner()).await?;
    Ok(Response::new(Pdf { content: res }))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let addr = env::var("SERVICE_ADDR_LATEX")
    .unwrap_or("[::1]:50081".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(LatexServer::new(LatexService))
      .serve_with_shutdown(addr, async {
        let _ = rx.await;
      })
      .await
      .unwrap()
  });

  tokio::signal::ctrl_c().await?;

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
