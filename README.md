# PGShield

PGShield is a performant PostgreSQL gateway, load balancer, connection pooler with caching and brokering capabilities, inspired by pgBouncer and pgPool2. 
It is designed to handle high request loads efficiently and provide high availability for PostgreSQL services. This project is written in Rust and leverages asynchronous I/O with Tokio.
I started learning Rust one year ago, driven by the need to build high-performance and safe systems. As I progressed, I wanted to create a tool that would be both challenging and beneficial to the community. 
PGShield is the result of this journey. It aims to address some of the issues users face with pgBouncer, providing an alternative with enhanced performance, caching, and brokering capabilities.
By leveraging Rust's safety and performance features, along with asynchronous I/O from Tokio, PGShield offers a robust solution for managing PostgreSQL connections at scale. 
Whether you are running a high-traffic website or a complex data service, PGShield can help ensure your database connections are handled efficiently and reliably.



## Features

- **Load Balancing**: Distributes incoming requests across multiple PostgreSQL servers.
- **Caching**: Caches connections and query results to reduce load on the database servers.
- **Brokering**: Manages database connections and redistributes them as needed.
- **High Availability**: Ensures that only healthy PostgreSQL servers are used to handle requests.
- **Asynchronous I/O**: Uses Tokio for efficient, non-blocking I/O operations.

## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/ilstarno/pgshield.git
    cd pgshield
    ```

2. **Build the project**:
    ```sh
    cargo build --release
    ```

3. **Run the project**:
    ```sh
    cargo run
    ```

### Windows Release

You can download the latest Windows release from the [Releases](https://github.com/ilstarno/pgshield/releases) page. Follow these steps to run PGShield on Windows:

1. **Download the latest release** from the Releases page.
2. **Extract the downloaded ZIP file** to a desired location.
3. **Open a command prompt** in the extracted directory.
4. **Run PGShield**:
    ```sh
    pgshield.exe
    ```

## Contributing
Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Configuration

PGShield uses a JSON configuration file to set up the PostgreSQL servers, cache settings, and other parameters. Below is an example `config.json`:

```json
{
    "db_hosts": ["host1:5432", "host2:5432"],
    "listen_port": "5433",
    "max_conns": 100,
    "cache_ttl": 600,
    "health_check_interval": 60,
    "replication_mode": true,
    "query_cache_ttl": 600,
    "database_discovery": true,
    "discovery_interval": 3600
}




