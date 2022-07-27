use tokio::net::{TcpListener, TcpStream};
use virtual_keyboard::VirtualKeyboard;
use std::io;

async fn process_socket(socket: TcpStream, vb: &VirtualKeyboard)  {
    // read data from TcpStream
    loop {
        socket.readable().await.unwrap();

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut buf = [0; 4096];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                vb.write_event(buf);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let vb = VirtualKeyboard::new();

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket, &vb).await;
    }
}
