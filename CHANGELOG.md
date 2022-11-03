## 0.1.3 -- 2022-11-02

- Fixed decoding the password hashing salt to match the official client (#1). This problem affects roughly half of accounts on cohost, assuming password salts are randomly distributed.

## 0.1.2 -- 2022-08-01

- Fixed pending attachment blocks to use an all-zeroes UUID instead of an empty string ([staff announcement of change](https://cohost.org/jkap/post/71976-potentially-breaking)).

## 0.1.1 -- 2022-07-31

- Added support for alt text for attachments.

## 0.1.0 -- 2022-07-31

- Initial release, with support for creating, sharing, editing, and deleting posts, as well as uploading attachments.
