# pgShield


pgShield is a performant PostgreSQL gateway, load balancer, connection pooler with caching and brokering capabilities, inspired by pgBouncer and pgPool2. 
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
    git clone https://github.com/pgshield/pgshield.git
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

You can download the latest Windows release from the [Releases](https://github.com/pgshield/pgshield/releases) page. Follow these steps to run PGShield on Windows:

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
  "postgresql_hosts": [
    {
      "host": "localhost:5432",
      "admin_auth_type": "trust"
    },
    {
      "host": "example.com:5432",
      "admin_auth_type": "password",
      "admin_username": "postgres",
      "admin_password": "mypassword"
    },
    {
      "host": "ldap.example.com:5432",
      "admin_auth_type": "ldap",
      "admin_username": "admin",
      "admin_password": "ldappassword"
    },
    {
      "host": "cert.example.com:5432",
      "admin_auth_type": "cert",
      "admin_username": "cert_user",
      "admin_password": "certpassword"
    }
  ],
  "listen_port": "8080",
  "max_conns": 100,
  "cache_ttl": 3600,
  "health_check_interval": 60,
  "replication_mode": false,
  "query_cache_ttl": 600,
  "logging": {
    "log_to_file": true,
    "log_to_console": true,
    "log_to_syslog": false,
    "log_dir": "/var/log/pgshield",
    "syslog_facility": "LOG_USER",
    "syslog_process_name": "pgshield"
  }
}




