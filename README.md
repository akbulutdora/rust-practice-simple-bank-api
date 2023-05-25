In this small project, I started from scaffold code to implement a JSON bank API to handle:

- `POST` to `/payments` - seller issues a payment
- `POST` to `/payments/PAYMENT_ID/refunds` - seller makes a (potentially partial) refund against a payment

The API uses the Postgres instance with docker.

Constraints: 
- Each time a `/payments` POST request is attempted, a new credit card number must be used.
- Payments will stored in an `integer` column in a DB.

There is a stubbed `Bank.Accounts.DummyService` to check whether a card holder has enough funds. This service would normally be located in a remote microservice.

### Running

```sh
# Adjust ports if 5432 is in use.
docker compose up -d

cargo install sqlx-cli

# This might yield errors but they'll be fixed in the next step
cargo build

cargo sqlx migrate run
```

The code can be tested with `cargo test`.

Initial documentation -> `cargo doc --no-deps --open`.

# /payments

The payments endpoint handles removing money from the customer's bank account. The rest is not our concern here.

It's not possible to edit or delete payments from the API.

## Accounts service

If the customer has enough funds on their account, this service will be used to place a hold. The hold will be relased or withdrawn depending on the business logic.

One of the constraints is to send as little requests to the Accounts service as possible. Note that these requests will not fail. There is no need for error handling on this side.

// TODO: I was here
## Creation

`POST`ing to `/api/payments` with valid data creates a new payment. Valid data is defined as follows:

- `amount`: a positive `integer` value representing the monetary amount that is requested from the client's account. It represents the amount in the EUR currency, in cents: a EUR 10.45 purchase would be encoded as an amount of `1045`.
- `card_number`: a 15-digit numerical-only `string` value containing the single-use credit card number to use for the purchase. This value is expected to be unique, and there's no need to consider this a particularly sensitive information as every card number can be assumed to no longer be usable as soon as it hits an API endpoint: the card number can be logged, stored in clear in the DB, etc.

The above attributes must be wrapped within a `payment` attribute:

```
{
  "payment": {
    "amount": 1045,
    "card_number": "123451234512345"
  }
}
```

If the provided attributes are valid and the payment has been created, a `201` HTTP status response will be returned along with the created payment in the body's "data":

```
{
  "data": {
    "id": "9decbf6d-c470-4a1f-ae7b-8fb2a39db318",
    "amount": 1045,
    "card_number": "123451234512345",
    "status": "approved"
  }
}
```

As you can see, the API will set an `id` value and the response will contain a "approved" status in addition to the 201 HTTP status.

A "show" endpoint is also exposed by the API, and will return the above response if a GET request is made to `/api/payments/9decbf6d-c470-4a1f-ae7b-8fb2a39db318`. It's presence is mainly for convenience, as it's implementation won't be part of this challenge.

### Unhappy paths

Invariants:

- there is at most one payment per `card_number`: payment creation requests for a given `card_number` already associated with a payment record should return a 422 status.

Unhappy paths originating from an unhappy response from the accounts service should respond with a body containing the same data as a successful response (see above), but with a differing payment "status" value and HTTP status, as follows:

- `insufficient_funds`: `402 Payment required` and payment status "declined"
- `invalid_account_number`: `403 Forbidden` and payment status "declined"
- `service_unavailable`: `503 Service unavailable` and payment status "failed"
- `internal_error`: `500 Internal error` and payment status "failed"

Sadly, the "accounts service" mentioned above is really fragile. We've had issues in the past where it was unable to handle the load, so we do our best to not send unnecessary requests and the following cases should NOT contact the accounts API:

- payment requests for negative amounts should return a 400 response
- payment requests for 0 should return a 204 response
- invalid card formats should return a 422 response

In each of the 3 cases, the API response should contain a body with a "declined" payment status that is similar to the successful response above:

```
{
  "data": {
    "id": "626becac-e2ff-4a7b-95e9-4c8d2e8025da",
    "amount": 0,
    "card_number": "123451234512345",
    "status": "declined"
  }
}
```

# /refunds

The refunds endpoint handles refunding all or part of the money a customer spent on a purchase: this money will later be transferred back to the customer, but that won't concern us here.

Since this is dealing with money, the API only exposes endpoints to create new refunds and view existing refunds: it's not possible to (e.g.) edit or delete refunds from the API.

## Creation

`POST`ing to `/api/payments/PAYMENT_ID/refunds` with valid data should create a new refund. Valid data is defined as follows:

- `amount`: a positive `integer` value representing the monetary amount that is to be refunded to the client's account. It represents the amount in the EUR currency, in cents: a EUR 2.90 refund would be encoded as an amount of `290`.

The above attributes must be wrapped within a `refund` attribute:

```
{
  "refund": {
    "amount": 290
  }
}
```

If the provided attributes are valid and the refund has been created, a `201` HTTP status response will be returned along with the created refund in the body's "data":

```
{
  "data": {
    "id": "a3828107-d407-45b0-86dc-eea7571df3a7",
    "amount": 290
  }
}
```

A "show" endpoint is also exposed by the API, and will return the above response if a GET request is made to `/api/payments/PAYMENT_ID/refunds/a3828107-d407-45b0-86dc-eea7571df3a7`. It's presence is mainly for convenience, as it's implementation won't be part of this challenge.

### Unhappy paths

Refund requests for negative amounts should return a 400 response (similar to the `/payments` endpoint).

The API will return a 404 response if the payment against which the refund is being attempted:

- doesn't exist
- has a status other than "approved"

There is no limit on the number of refunds made against a payment: as long as the sum of the refund amounts never exceeds the payment amount, all is well. In other words, all of these scenarios are valid:

- full refund

  1. payment for 10_00
  1. refund for 10_00

- partial refund

  1. payment for 10_00
  1. refund for 2_00
  1. refund for 5_00

- partial refunds up to payment amount
  1. payment for 10_00
  1. refund for 2_00
  1. refund for 5_00
  1. refund for 3_00

The following scenarios, however, would fail with the last refund request listed:

- full refund with excessive amount

  1. payment for 10_00
  1. refund for 11_00

- partial refund with excessive amount
  1. payment for 10_00
  1. refund for 2_00
  1. refund for 9_00

The failed refund attempt would return a 422 status.