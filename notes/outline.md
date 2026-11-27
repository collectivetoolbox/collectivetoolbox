

Outline components microservices

Paid:
- net relay: server that passes traffic from authenticated clients to a netlayer a la spritely(?) or tailscale; probably client library that makes it easy to use securely? The usual security considerations would apply as opening sockets or ports on an untrusted network connection
  - ^ this may not be needed, or maybe not in that form. Multiple instances will need to be able to pass messages between each other somehow for collaboration, and presumably for security you don't want to have them *directly* communicate.
- storage api for backups, like backblaze, and client libraries to make it easy to use it securely (plug and play secure backups for a given folder or in-browser storage prefix). Would probably run separately for each client, and then they’d synchronize with each other over higher level protocols.
- Possibly: STUN and TURN servers , not needed for local mvp, probably only for authenticated clients, relay traffic between NATed webrtc peers. Local and in browser webrtc library? Might need some sort of peer negotiation api to figure out which peers are available to connect to, assign them names, and allow establishing a connection with a given peer given its name. Probably something like a “room” in zoom where peers would join the room by the room name and then it would act like a pool of candidate peers to connect to. After establishing connection, would want some sort of communication system though that may be out of scope for webrtc client. (Possibilities: bidirectional message passing, sockets, http server, webtorrent, etc? Not sure what the best way to communicate over webrtc is - update: found some potential libraries; possibly either matchbox (not very well maintained) or veilid (actively developed but incomplete)? iroh also tries to solve this problem but I looked at it briefly and seem to remember it had some serious deficiency that made it not practically usable)
- insert permissions: not for read-only mvp, but some sort of metered get-a-key system to allow storing new common library nodes [Have a work-in-progress implementation of this now, as there's no way to fill in the common library without it]

Would those three plus v86 be enough to use the web as an equivalent to a native desktop application platform?

Local mvp

- Filesystem api. Just an abstraction over the native filesystem and browser local storage to start.
- ui server: http server that opens in the native web browser. Plain html pages with links that post from one to the next.
- ability to create local nodes, including relationship nodes and relationship type nodes.
- history page to view both applied and withdrawn attestations for a node.
- For a normal document, editing would work by creating a node to represent the document, then subsequent edits would be separate nodes that contain revisions of it. The document could then be accessed from the original node id and the latest revision would be shown-this requires an index recording the current state of the graph as computed from all valid attestations.

Later on features:

- the same model could be applied over the network: attestations could be signed by an identity; trusted attestations would be treated as valid. That would require the index to be fragmented across multiple untrusted peers.
- ability to copy nodes from one graph to another. Would need a way to copy with or without history, default to without- just applying the current state of all trusted attestations.
- local documents could depend on network ones, or ones shared from another client’s private graph, but network nodes in the common library could not depend on local documents.
- security 🙃
- Indexer: would index relationships between nodes (and derive relationships for things like relating documents to words in them for full-text search). Each node would get its own index of relationships for it, and those indices would be shared between computers up to a different date or number of relationships. So for instance the node for the word "the" might have a million related nodes in one index chunk, a million in another, etc. (or maybe it would be a stop word, but that's just for an example.) To add a new node, a new index record would need to be written, so the last small chunk of the index for the each of the related nodes would have to get rewritten.