# Queuetify

Create a Spotify session where multiple users can queue and vote for songs

## Local deployment
Follow these instructions to register the application, which will provide you with a
*Client ID* and *Client Secret*. You also need to add the following redirect URI in
your application settings: *http://localhost:8080/callback*

Add a file named *.env.secret* in the server directory and add the following variables
with the values found in your application dashboard:

```
QUEUETIFY_APP_SPOTIFY__CLIENT_ID=<Client ID>
QUEUETIFY_APP_SPOTIFY__CLIENT_SECRET=<Client Secret>
QUEUETIFY_APP_SPOTIFY__REDIRECT_URI="http://localhost:8080/callback"
```

To deploy, run:
```
make serve
```

To tear down deployment, run:
```
make down
```

> **_Note:_** All new Spotify third-party applications begin in Development Mode. Users of the app then needs to be managed, see: https://developer.spotify.com/community/news/2021/05/27/improving-the-developer-and-user-experience-for-third-party-apps/