# TODOs

- Consider if we can optimise is_admin and oldest_active_character to not be woven
  through the whole route tree. Options:
  - Get that data in some other way, like a client side fetch
  - Allow handlers to return an enum over `response` and `template` instead of
    always rendering the template at the bottom level. This would allow the top-level
    route to inject the universally used data (assuming the templates implement a
    trait for that) and rendering only thereafter.
- Look over the user-facing error messages
