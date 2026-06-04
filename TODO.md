# TODOs

- Consider if we can optimise is_admin and oldest_active_character to not be woven
  through the whole route tree. Options:
  - Get that data in some other way, like a client side fetch
  - Allow handlers to return an enum over `response` and `template` instead of
    always rendering the template at the bottom level. This would allow the top-level
    route to inject the universally used data (assuming the templates implement a
    trait for that) and rendering only thereafter.
- Look over the user-facing error messages


- [DONE] Add a BP changes table, to track amaranth increases and reductions through ritual or torpor.
- [DONE] Make the character presentation show torpor time in years, months and days (and not print a unit when value is 0)
- [DONE] In /admin/character make the character name link into the character, same as the ID currently.
- [DONE] Fix the stat-sum queries so that physical/mental/organizational ability all begin at 6 without any raises.
