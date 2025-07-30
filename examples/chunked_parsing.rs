//! Simple async chunked parsing example demonstrating core API usage

use serde::Deserialize;
use serde_ply::chunked_header_parser;
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Deserialize, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Chunked PLY Parsing ===");

    // Demo 1: Basic async parsing from memory
    demo_basic_async_parsing().await?;

    // Demo 2: Network stream simulation
    demo_network_parsing().await?;

    Ok(())
}

async fn demo_basic_async_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Basic Async Parsing ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 4
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0 6.0
7.0 8.0 9.0
10.0 11.0 12.0
"#;

    let mut async_reader = Cursor::new(ply_data.as_bytes());

    // Step 1: Parse header from async source chunk by chunk
    let mut header_parser = chunked_header_parser();
    loop {
        let mut buffer = vec![0u8; 64]; // Small chunks to show chunking
        let bytes_read = async_reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Err("EOF before header complete".into());
        }
        buffer.truncate(bytes_read);

        if header_parser.parse_from_bytes(&buffer)?.is_some() {
            println!("Header parsing complete");
            break;
        }
    }

    // Step 2: Create element parser (inherits leftover data automatically)
    let mut parser = header_parser.element_parser::<Vertex>("vertex")?;
    let mut total_vertices = Vec::new();

    // Step 3: Continue reading chunks for element data
    loop {
        let mut buffer = vec![0u8; 32];
        let bytes_read = async_reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break; // EOF
        }
        buffer.truncate(bytes_read);

        if let Some(vertices) = parser.parse_from_bytes(&buffer)? {
            println!("Parsed {} vertices", vertices.len());
            total_vertices.extend(vertices);
        }

        tokio::task::yield_now().await;

        if parser.is_complete() {
            break;
        }
    }

    println!("Total: {} vertices", total_vertices.len());
    assert_eq!(total_vertices.len(), 4);

    Ok(())
}

async fn demo_network_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Network Stream Parsing ---");

    // Generate binary PLY data
    let binary_data = generate_binary_ply(1000);
    let mut network_stream = SimulatedNetworkStream::new(binary_data);

    // Parse header chunk by chunk
    let mut header_parser = chunked_header_parser();
    loop {
        let mut buffer = vec![0u8; 512];
        let bytes_read = network_stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Err("EOF before header complete".into());
        }
        buffer.truncate(bytes_read);

        if header_parser.parse_from_bytes(&buffer)?.is_some() {
            break;
        }
    }

    let mut parser = header_parser.element_parser::<Vertex>("vertex")?;
    let mut total_vertices = 0;

    // Parse from network stream with variable chunk sizes
    loop {
        let mut buffer = vec![0u8; 4096];
        let bytes_read = network_stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        buffer.truncate(bytes_read);

        if let Some(vertices) = parser.parse_from_bytes(&buffer)? {
            total_vertices += vertices.len();
            println!(
                "Network chunk: {} vertices (total: {})",
                vertices.len(),
                total_vertices
            );
        }

        if parser.is_complete() {
            break;
        }
    }

    println!("Network parsing complete: {} vertices", total_vertices);
    Ok(())
}

// Helper functions

fn generate_binary_ply(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {}
property float x
property float y
property float z
end_header
"#,
        vertex_count
    );

    let mut binary_data = header.into_bytes();

    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        binary_data.extend_from_slice(&base.to_le_bytes());
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes());
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes());
    }

    binary_data
}

// Simple network stream that returns variable chunk sizes
struct SimulatedNetworkStream {
    data: Vec<u8>,
    position: usize,
    chunk_sizes: Vec<usize>,
    chunk_index: usize,
}

impl SimulatedNetworkStream {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
            chunk_sizes: vec![1024, 2048, 512, 4096], // Variable sizes
            chunk_index: 0,
        }
    }
}

impl AsyncRead for SimulatedNetworkStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.position >= self.data.len() {
            return std::task::Poll::Ready(Ok(()));
        }

        let chunk_size = self.chunk_sizes[self.chunk_index % self.chunk_sizes.len()];
        let available = std::cmp::min(chunk_size, buf.remaining());
        let end = std::cmp::min(self.position + available, self.data.len());
        let bytes_to_copy = end - self.position;

        if bytes_to_copy > 0 {
            buf.put_slice(&self.data[self.position..end]);
            self.position = end;
            self.chunk_index += 1;
        }

        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_async_parsing() {
        let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0 6.0
"#;

        let mut async_reader = Cursor::new(ply_data.as_bytes());

        // Parse header chunk by chunk
        let mut header_parser = chunked_header_parser();
        loop {
            let mut buffer = vec![0u8; 32];
            let bytes_read = async_reader.read(&mut buffer).await.unwrap();
            if bytes_read == 0 {
                panic!("EOF before header complete");
            }
            buffer.truncate(bytes_read);

            if header_parser.parse_from_bytes(&buffer).unwrap().is_some() {
                break;
            }
        }

        let mut parser = header_parser.element_parser::<Vertex>("vertex").unwrap();
        let mut all_vertices = Vec::new();

        // Continue reading chunks for element data
        loop {
            let mut buffer = vec![0u8; 16];
            let bytes_read = async_reader.read(&mut buffer).await.unwrap();
            if bytes_read == 0 {
                break;
            }
            buffer.truncate(bytes_read);

            if let Some(vertices) = parser.parse_from_bytes(&buffer).unwrap() {
                all_vertices.extend(vertices);
            }

            if parser.is_complete() {
                break;
            }
        }

        assert_eq!(all_vertices.len(), 2);
    }
}
