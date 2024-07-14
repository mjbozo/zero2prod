# Create App
To create app, or recreate after destroying an existing one, run:
`doctl apps create --spec spec.yaml`

This will create the app on the DigitalOcean platform. It will take some time to build the project and provision
a Postgres database.


# Update App
If there is an existing app running, and there is a change to the deployment spec.yaml file, it can be updated using:
`doct apps update APP-ID --spec=spec.yaml`

The APP-ID can be found by running:
`doctl apps list`

> Note: DigitalOcean database 'Trusted Sources' must be disabled to update spec via command line.


# Migration
Once the app is creation and deployed, and the database is provisioned, we can run the database migration with:
`DATABASE_URL=DIGITAL-OCEAN-DB-CONNECTION-STRING sqlx migrate run`

The database connections string can be found on the DigitalOcean dashboard under
Settings > newsletter > Connection Details

> Note: DigitalOcean database 'Trusted Sources' must be disabled to perform database migration.
