## 0.2.0 -- 2023-07-31

- eggbug-rs is now "lightly maintained": pull requests will generally be merged without testing, and new releases will generally be "breaking" (e.g. 0.2.x -> 0.3.x) unless I am positively certain a change does not break semver
- Add ability to fetch posts by project and page (@NoraCodes, [#2](https://github.com/iliana/eggbug-rs/pull/2))
- Update attachment start API (@jkap, [#3](https://github.com/iliana/eggbug-rs/pull/3))
- Add read-only support for asks (@NoraCodes, [#4](https://github.com/iliana/eggbug-rs/pull/4))
- Audio attachments (@xenofem, [#6](https://github.com/iliana/eggbug-rs/pull/6))

## 0.1.3 -- 2022-11-02

- Fixed decoding the password hashing salt to match the official client ([#1](https://github.com/iliana/eggbug-rs/issues/1)). This problem affects roughly half of accounts on cohost, assuming password salts are randomly distributed.

## 0.1.2 -- 2022-08-01

- Fixed pending attachment blocks to use an all-zeroes UUID instead of an empty string ([staff announcement of change](https://cohost.org/jkap/post/71976-potentially-breaking)).

## 0.1.1 -- 2022-07-31

- Added support for alt text for attachments.

## 0.1.0 -- 2022-07-31

- Initial release, with support for creating, sharing, editing, and deleting posts, as well as uploading attachments.
