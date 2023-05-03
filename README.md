# backer-upper
A CLI tool to manage backups

## Usage
backer-upper uses [gpg](https://gnupg.org/) for encryption, so you will need that installed and a key generated if you want to encrypt your backups. You can generate a key with this command:

```sh
gpg --full-generate-key
```

## Sync files
The `sync` command requires a specially formatted TOML file that describes a number of backups and their schedules. The `sync` command consults this file and the last backup performed to determine if it's time for a new backup, and will also delete old redundant backups if required.

The TOML file should be structured like this:
```toml
[name-of-backup]
globs = ["/files/to/back/up", "/more/files/to/back/up/*"]
gpg_id = "backup@backup.backup" # Optional
host = "my.remote.host" # Optional
dir = "/backup/dir/"
format = "backup_%Y-%m-%d_%H:%M:%S.tar.gz.gpg"
interval = "1 day"
copies = 3 # Optional
```

If `gpg_id` is specified, then the GPG key with that ID will be used to encrypt the backup. Keep in mind that you will need that key to restore the backup.

The `host` field is only necessary if backups are kept on a remote host. If it is specified, `ssh` and `scp` are used to upload the backups automatically.

You can use any valid date format string for `format`. Do not include any path separators, `dir` should point directly to the directory containing all the backup files.

`interval` always looks like `{integer} {unit}`. A variety of intervals are accepted:
* "M", "month", "months"
* "w", "week", "weeks"
* "d", "day", "days"
* "h", "hour", "hours"
* "m", "minute", "minutes"
* "s", "second", "seconds"

`copies` determines how many old backups to keep. If there are too many, the oldest is deleted. If copies is not specified, then old backups are never deleted.

If you would like to keep redundant backups on different timescales (i.e. 24 hourly backups, 7 daily backups, 4 weekly backups, and 12 monthly backups), then you should set up multiple sections, each with their own schedule. There will be some redundancy with the most recent backup, but that is the price you pay for simplicity.
