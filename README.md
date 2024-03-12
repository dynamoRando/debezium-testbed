# Overview
A Docker testbed for rapidly cloning a MySQL database and plugging in a Debezium connector for that cloned database. 

## Requirements
Requires Docker.

# Fixtures

The templates for the MySQL and Debezium connector are in:

- `dbz_init/dbz.json`
- `sql_init/test.sql`

These two files are the basis for your testbed.

By default, the database created will be named `mydb` unless otherwise specified. You can also replace the const

```rust
const DEFAULT_MYSQL_DB: &str = "mydb";
```

with the database name of your choice if you'd like. 

## MySQL 
For MySQL, the user is `root` and the password is `testbed`.

## Docker network
All containers created by this testbed are part of the `testbed` Docker network.

# Startup
On startup, this creates the following Docker stack with the network `testbed` 

- Zookeeper
- Kafka
- Schema Registry
- Kafka Connect
- Kafka UI
- MySQL

It also will download into the Kafka Connect container the MySQL drivers and Debezium plugin. It also creates the MySQL db in the `sql_init/test.sql` directory in the MySQL container.

Prior to starting, it will attempt to stop and remove any of the above containers if they were already created.

# Shutdown
On shutdown this will attempt to stop all running containers, remove them, and prune any unused volumes.

# How this works
This repo creates an HTTP server with two endpoints:

```
- POST: /testbed/<name>
- GET: /shutdown
```

Sending a `POST` to the above endpoint will:
- Create a new named testbed with `<name>_<random-number>`. `<name>` in this case may be the name of a test you wish to run, for example.
- Clone the `mydb` MySQL database by doing a `mysqldump DEFAULT_MYSQL_DB | mysql <name>` in the MySQL container, where `<name>` is the generated name in the previous step.
- Clone the Debezium connector settings in `dbz_init/dbz.json`; replacing any of the default `mydb` names with the `<name>` value.
- Responds with the following JSON:

```rust
pub struct TestBed {
    pub name: String,
}
```

where `name` is the testbed you can:

- Perform MySQL INSERT/UPDATE/DELETE statements against
- Poll the generated Debezium topic for information

# Demo
For a demo of how all this works, start by in a new terminal running `cargo run`. This will bring up the HTTP Server along with the Docker stack described. The HTTP server is implemented in Rocket, starting on `http://localhost:8000`.

Once the HTTP server is up an running, give it a few moments to finish installing the MySQL drivers in Kafka Connect and Debezium.

In another terminal, navigate to the `/demo` directory and run `cargo test`. This will make an HTTP call to `http://localhost:8000/testbed/demo`; which returns a randomly generated `demo_<number>` for the test to use.

The test fires off a thread to poll Kafka for the appropriate topic for an `example` table. The test also inserts numbers into the `example` table; while trying to poll the Kafka topic to make sure data is being injected into it. If data is found, the test passes.

The hope is that you can quickly re-run tests over and over, allowing for quick iteration, getting a fresh database and kafka topic to work with each time.

If you want to shut down the server gracefully, you can navigate in your browser to [http://localhost:8000/shutdown](http://localhost:8000/shutdown) which will stop and remove all containers and then prune unused volumes.

Kafka UI should be available at [http://localhost:8099](http://localhost:8099) if you want to inspect your environment.

# Notes
- May need to add `127.0.0.1 kafka` to your `/etc/hosts` file for this stack to work.
- The MySQL init file is bind mounted to the MySQL container; using the full path.