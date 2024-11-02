## Deploying

We use shuttle to deploy this.

Deploying: `shuttle init` if you haven't ever done it before

Then `shuttle deploy`

## Database

To create a migration:

`sqlx migrate add <name>`

To run migrations:

`sqlx migrate run`