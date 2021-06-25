use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;


#[tokio::main]
async fn main() {
    let host = "localhost:8080" ;

    let listener = TcpListener::bind(host)
        .await
        .expect("Binding failed") ;

    let (tx, _rx) = broadcast::channel(10) ;

    loop {
        let (mut socket, addr) = listener.accept()
            .await
            .expect("Accept failed");

        let tx = tx.clone() ;
        let mut rx = tx.subscribe() ;

        tokio::spawn( async move {

            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {

                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break ;
                        }
                        tx.send((line.clone(), addr)).expect("Error sending") ;
                        line.clear() ;
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.expect("Failed to receive message") ;

                        if addr != other_addr {
                            writer.write_all(msg.as_bytes())
                                .await
                                .expect("Failed to write back to client");
                       }
                    }

                }

            }
        });
    }

}
