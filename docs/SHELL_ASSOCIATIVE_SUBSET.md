# First associative-map subset

Implementation in progress, checked 2026-11-04 against GNU Bash 5.3 primary Arrays and declare documentation. Fixed real-Bash comparisons and lesson acceptance are recorded separately; this document defines supported behavior, not full Bash conformance.

An associative array is a lookup from a text key to one text value. It is a Bash feature, not portable POSIX shell syntax. This original Rust implementation is entirely in-process; it imports no Bash code and never invokes a host shell for learner work.

## Supported forms

| Action | Form | Meaning |
| --- | --- | --- |
| Declare a fresh named map | `declare -A routes` | Start an empty map outside functions; redeclaring the same map retains its entries. |
| Store a literal key | `routes[oak]='north wing'` | Store or replace one value; spaces are part of the value. |
| Store a key supplied as text | `routes["$key"]="$value"` | Expand one ordinary scalar key; keep its resulting bytes as data. |
| Read a key | `"${routes[oak]}"` or `"${routes[$key]}"` | Read one value; missing keys read as empty text without inserting an entry. |
| Count entries | `"${#routes[@]}"` | Count assigned keys, including keys whose value is empty. |

Literal keys may be quoted; dynamic keys use one ordinary scalar name. Keep map reads quoted. A missing value and a present empty value both print empty text; the count distinguishes their effect on the map. There is no general membership operator in this slice. Keys 01 and 1 are distinct strings. Plain `$routes` and ordinary scalar assignment/read/for target `routes` refer to key 0 and retain other entries.

A function may update an existing global map. Command-capture and pipeline bindings remain isolated according to the existing simulator policy; file effects retain their documented scope. This does not model alternative Bash lastpipe settings.

## Honest limits

Only standalone element assignments are supported. Calculate command-capture results in a separate scalar before assigning to a map. No compound initializer, enumeration/order promise, attributes, unset, scalar/indexed conversion, function-local map declaration/shadowing, command substitution in keys or map assignment values, composed/complex subscripts, special @/* keys, map exports/environment observation or map/xtrace combination. These forms produce an explicit diagnostic; they are not simulated as successful. Other ordinary indexed-array syntax retains its previous grammar.

Each map has at most 128 entries; each nonempty key has at most 128 UTF-8 bytes. Empty values are valid. Keys and values plus conservative node overhead participate in the existing 256 KiB memory budget, including snapshots and pending input. Existing instruction/output/expansion limits and cooperative Stop remain in force. Expanded key data is never reparsed as commands, arithmetic or paths. Learner scripts see only simulated files and state.

Primary sources: [GNU Bash Arrays](https://www.gnu.org/software/bash/manual/html_node/Arrays.html), [declare builtin](https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html). Installed primary manual edition 5.3 was read; the GNU single-page manual was retrieved on 2026-11-04 after split-page fetch timeout. No source text or implementation was copied. Fixed comparison observations, unsupported/negative tests and the actual18 compatibility baseline are separate evidence.
