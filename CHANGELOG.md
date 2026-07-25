While I won't document all breaking changes until this application is stable, I'll try to note the most significant ones here.

# July 19, 2026

Breaking changes:

- Removed function from `src/utilities/utilities.rs`: `strtohex<T>(s: T) -> String`.
  - Drop-in replacement: `pub fn bin2hex<T>(s: T) -> String`.
- Removed function from `src/utilities/utilities.rs`: `vectohex<T>(s: T) -> String`.
  - Drop-in replacement: `pub fn bin2hex<T>(s: T) -> String`.

# Earlier:

- Changed the database format to use Turso rather than redb. This migration is not automatic, so any existing database is not carried over. I'm assuming no one else is using this feature yet, so I didn't bother including a migration tool (yet). If you did have a database in the old format, my apologies, please contact [info@collectivetoolbox.com](mailto:info@collectivetoolbox.com) and I'll prioritize adding a tool to import those files.
