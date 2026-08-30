While I won't document all breaking changes until this application is stable, I'll try to note the most significant ones here.

# August 30, 2026

- Relocated Dcs 286 `Begin set text color` and 287 `End set text color` (introduced in commit b41c5, Feb 2019) to 304 `Begin set text color` and 305 `End set text color`, as those IDs were already assigned to Dcs 286 `Begin italicized text` and 287 `End italicized text`. It looks like all files I have use those Dc IDs for italics, rather than for coloring.

# August 20, 2026

Breaking changes:
- Reworked API of `ctb-formats-encoding`.

# August 1, 2026

Breaking changes:

- Removed `ExpectWithErr` trait in favor of more graceful error handling. As it wouldn't be detected by the `expect_used` lint, it was something of a footgun.

# July 19, 2026

Breaking changes:

- Removed function from `src/utilities/utilities.rs`: `strtohex<T>(s: T) -> String`.
  - Drop-in replacement: `pub fn bin2hex<T>(s: T) -> String`.
- Removed function from `src/utilities/utilities.rs`: `vectohex<T>(s: T) -> String`.
  - Drop-in replacement: `pub fn bin2hex<T>(s: T) -> String`.

# Earlier:

- Changed the database format to use Turso rather than redb. This migration is not automatic, so any existing database is not carried over. I'm assuming no one else is using this feature yet, so I didn't bother including a migration tool (yet). If you did have a database in the old format, my apologies, please contact [info@collectivetoolbox.com](mailto:info@collectivetoolbox.com) and I'll prioritize adding a tool to import those files.
