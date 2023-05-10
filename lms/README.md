# Sample Domain-Driven Library-Management-System MicroService

## Install 
See https://www.cargo-lambda.info/guide/getting-started.html and https://github.com/awslabs/aws-lambda-rust-runtime

```bash
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

## Local AWS DDB Unit-Testing
In order to run unit-tests locally, See 
- https://docs.aws.amazon.com/sdk-for-rust/latest/dg/dynamodb-local.html 
- https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.DownloadingAndRunning.html#docker.

And define local profile

```
[profile localstack]
region = us-east-1
aws_access_key_id = AKIDLOCALSTACK
aws_secret_access_key = localstacksecret
```

Then start DDB Docker
```
docker-compose -f ddb-docker-compose.yaml up
```

## Local Lambda Testing

### Testing with SAM (See https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-using-debugging.html)

#### Starting lambda endpoint 
export SAM_CLI_TELEMETRY=0
```
sam local start-lambda
```

#### Invoke a function locally in debug mode on port 5858
```
sam local invoke -d 5858 <function logical id>
```

#### Start local API Gateway in debug mode on port 5858
```
sam local start-api -d 5858
```

### Watching Lambda locally

```bash
cargo lambda watch
cargo lambda invoke --data-ascii "{ \"body\": \"hi\" }"
```
or
```
sam local invoke "Ratings" -e event.json
```

## Building Lambda Functions (You must do this before CDK deployment)
```bash
cargo lambda build --release
```

## Deploy
Though, you can deploy lambda functions with cargo but we will use CDK (https://aws.amazon.com/cdk/) for all infra deployments.  See ../cdk folder.
```bash
cargo lambda deploy
```

## Lint
```
rustup update
rustup component add clippy
cargo clippy
```

## Testing Lambda APIs Locally

### Start locally

```
docker-compose -f ddb-docker-compose.yaml up
```

### Start Lambda locally
```bash
cargo lambda watch
or
cargo lambda watch --only-lambda-apis #if you run lambda main from IDE or CLI
```

### Build
```bash
cargo build --release
```

### Testing catalog Lambdas
Add a book
```bash
curl -H "Content-Type: application/json" http://localhost:9000/catalog -d '{"isbn": "123", "title": "my book"}'
```
which would return something like:
```json
{
  "book": {
    "dewey_decimal_id": "749",
    "book_id": "a2b25506-2948-47bb-9c4a-cf9ad480c10b",
    "version": 0,
    "author_id": "623a01ca-8ba9-41cd-b8b6-85a5711f8453",
    "publisher_id": "f0cff296-9f6e-4b25-95e1-a783661bf91f",
    "language": "en",
    "isbn": "123",
    "title": "my book",
    "book_status": "Available",
    "restricted": false,
    "published_at": "2023-05-09T20:55:56.073008+00:00",
    "created_at": "2023-05-09T20:55:56.073027+00:00",
    "updated_at": "2023-05-09T20:55:56.073027+00:00"
  }
}
```
Finding the book by id
```bash
curl -H "Content-Type: application/json" http://localhost:9000/catalog/f58ef32a-6f24-4314-8782-c7ebcad0ab59
```
that returns
```json
{
  "book": {
    "dewey_decimal_id": "220",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "version": 0,
    "author_id": "4c24b180-a146-410a-b68c-9d83c57adebc",
    "publisher_id": "88b47029-cad1-443f-8d67-aaf13863e924",
    "language": "en",
    "isbn": "123",
    "title": "my book",
    "book_status": "Available",
    "restricted": false,
    "published_at": "2023-05-09T22:18:25.436359+00:00",
    "created_at": "2023-05-09T22:18:25.436366+00:00",
    "updated_at": "2023-05-09T22:18:25.436371+00:00"
  }
}
```

### Testing patrons Lambdas
Add a patron
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/patrons -d '{"email": "test-email@xyz.com"}'
```
that returns:
```json
{
  "patron": {
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "version": 0,
    "first_name": "",
    "last_name": "",
    "email": "test-email@xyz.com",
    "under_13": false,
    "group_roles": [],
    "num_holds": 0,
    "num_overdue": 0,
    "home_phone": null,
    "cell_phone": null,
    "work_phone": null,
    "street_address": null,
    "city": null,
    "zip_code": null,
    "state": null,
    "country": null,
    "created_at": "2023-05-09T22:20:28.898831",
    "updated_at": "2023-05-09T22:20:28.898833"
  }
}
```

Getting patron:
```bash
curl -H "Content-Type: application/json" http://localhost:9000/patrons/cf49007e-e7fa-42c3-ac56-e15b9530597e|jq '.'
```
that returns:
```json
{
  "patron": {
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "version": 0,
    "first_name": "",
    "last_name": "",
    "email": "test-email@xyz.com",
    "under_13": false,
    "group_roles": [],
    "num_holds": 0,
    "num_overdue": 0,
    "home_phone": "",
    "cell_phone": "",
    "work_phone": "",
    "street_address": null,
    "city": null,
    "zip_code": null,
    "state": null,
    "country": null,
    "created_at": "2023-05-09T22:21:35.142750",
    "updated_at": "2023-05-09T22:21:35.142757"
  }
}
```

### Checkout book Lambda
Checkout a book:
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/checkout -d '{"patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e", "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59"}'|jq
```

that returns:
```json
{
  "checkout": {
    "checkout_id": "4a7ea5c5-939d-4934-8715-071c7ab5bc71",
    "version": 0,
    "branch_id": "dev",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "checkout_status": "CheckedOut",
    "checkout_at": "2023-05-09T22:36:55.162807+00:00",
    "due_at": "2023-05-24T22:36:55.162808+00:00",
    "returned_at": null,
    "created_at": "2023-05-09T22:36:55.162812+00:00",
    "updated_at": "2023-05-09T22:36:55.162812+00:00"
  }
}
```

Returning a book
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/checkout/return -d '{"patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e", "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59"}'
```
that returns:
```json
{
  "checkout": {
    "checkout_id": "6b432212-8136-45a5-a8c4-953da73ee24f",
    "version": 0,
    "branch_id": "dev",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "checkout_status": "Returned",
    "checkout_at": "1970-01-01T00:00:00+00:00",
    "due_at": "2023-05-09T22:36:59.145408+00:00",
    "returned_at": "2023-05-09T22:36:59.145607",
    "created_at": "2023-05-09T22:36:59.145415+00:00",
    "updated_at": "2023-05-09T22:36:59.145421+00:00"
  }
}
```

### Hold book Lambda
Hold a book
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/hold -d '{"patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e", "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59"}'
```
that returns
```json
{
  "hold": {
    "hold_id": "b6cbff12-fe0b-4be0-9566-5e221e52c8c5",
    "version": 0,
    "branch_id": "dev",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "hold_status": "OnHold",
    "hold_at": "2023-05-09T22:38:52.905822+00:00",
    "expires_at": "2023-05-24T22:38:52.905822+00:00",
    "canceled_at": null,
    "checked_out_at": null,
    "created_at": "2023-05-09T22:38:52.905825+00:00",
    "updated_at": "2023-05-09T22:38:52.905825+00:00"
  }
}
```

Canceling a hold
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/hold/cancel -d '{"patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e", "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59"}'
```
```json
{
  "hold": {
    "hold_id": "b6cbff12-fe0b-4be0-9566-5e221e52c8c5",
    "version": 0,
    "branch_id": "dev",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "hold_status": "Canceled",
    "hold_at": "2023-05-09T22:39:51.920045+00:00",
    "expires_at": "2023-05-09T22:39:51.920052+00:00",
    "canceled_at": "2023-05-09T22:39:51.920078",
    "checked_out_at": null,
    "created_at": "2023-05-09T22:39:51.920058+00:00",
    "updated_at": "2023-05-09T22:39:51.920063+00:00"
  }
}
```

Checking out a hold book
```bash
curl -v  -H "Content-Type: application/json" http://localhost:9000/hold/checkout -d '{"patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e", "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59"}'
```
that returns
```json
{
  "hold": {
    "hold_id": "f5fdb835-5ea2-428d-af12-a81ffb1b3f35",
    "version": 0,
    "branch_id": "dev",
    "book_id": "f58ef32a-6f24-4314-8782-c7ebcad0ab59",
    "patron_id": "cf49007e-e7fa-42c3-ac56-e15b9530597e",
    "hold_status": "CheckedOut",
    "hold_at": "2023-05-09T22:40:54.705417+00:00",
    "expires_at": "2023-05-09T22:40:54.705424+00:00",
    "canceled_at": null,
    "checked_out_at": "2023-05-09T22:40:54.705443",
    "created_at": "2023-05-09T22:40:54.705430+00:00",
    "updated_at": "2023-05-09T22:40:54.705435+00:00"
  }
}
```
