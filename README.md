# Shifty backend

This is the backend of the shift planning tool shifty.

## Build and run
You will need the *sqlx-cli* installed in order to prepare the database.

First create the *.env* file.  You may use the *env.example* file as a template.

Then prepare your local sqlite database: `sqlx setup --source migrations/sqlite`

Now you can run: `cargo run` or `cargo watch -x run` to run the backend. 

## License
This project is free and open source and double licensed under

* MIT License
* Apache License 2.0.

Dual license means you are free to pick one or both licenses as this is the de-facto standard in the Rust ecosystem.
