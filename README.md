# withings-api

This is UNOFFICIAL rust client to acess Withings API.
More Withings API document can be found in [Withings developer documentation](https://developer.withings.com/).

## Supported API

- [OAuth 2.0](https://developer.withings.com/api-reference#tag/oauth2)
- [Measure - Getmeas](https://developer.withings.com/api-reference#operation/measure-getmeas)

## Getting started

### Register your app

[Register your app](https://oauth.withings.com/) and get your client ID and consumer secret.

### Setup .env file

Rename `dot.env.sample` to `dot.env` in your working directory.

```
CLIENT_ID="client ID"
CONSUMER_SECRET="consumer secret"
CALLBACK_URL=https://localhost
CODE="You can get it by login to the get_authorize_url URL." 
ACCSESS_TOKEN="You can get it by get_access_token."
```

## Example

### `get_authorize_url`

```
> cargo run --example get_authorize_url --features=env
```


### `get_access_token`
```
> cargo run --example get_access_token --features=env
```


### `get_meas`
```
> cargo run --example getmeas --features=env
```


