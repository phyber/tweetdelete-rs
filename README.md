# TweetDelete

A small utility to quickly remove old Tweets from your timeline.

## Configuration

Your configuration should be located at `~/.tweetdelete.yaml`. At a minimum,
you will need the `api` configuration filled in with the various API keys from
[Twitter].

```yaml
---
api:
  access_token_key: 'EXAMPLE'
  access_token_secret: 'EXAMPLE'
  consumer_key: 'EXAMPLE'
  consumer_secret: 'EXAMPLE'

general:
  log_file: '~/.local/var/log/tweetdelete.log'
  max_tweet_age: 180
```

## Running

Once the correct configuration is in place, it should just be a case of running
`tweetdelete`. You may wish to run in `--dry-run` mode to begin with, so you
don't accidentally delete Tweets while you didn't mean to.

<!-- links -->
[Twitter]: https://developer.twitter.com/
