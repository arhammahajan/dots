# Writing calibration

These are invented teaching examples, not excerpts from the reference blogs or claims about the author's projects. Their topics are interchangeable; use the current draft to choose what to explain. Do not reproduce their wording, outline, or sentence patterns as a template.

## Fresh composition

The default task is to write anew from a content brief extracted from the draft. Do not retain its worst prose in the name of preserving voice. References to preserving voice in earlier feedback mean preserving the author's perspective and factual identity, not keeping rough sentences or mirroring their paragraph structure.

Source:

> now this is where the problem started to become a problem. both requests checked name free. both inserted. the check was fine until it wasn't. added unique constraint in db. no benchmarks yet.

Freshly composed:

> Checking whether a name is available before inserting it leaves a gap in which another request can do the same thing. Both requests can see the name as available, then both go ahead with their inserts, so the availability check doesn't enforce the uniqueness I need.
>
> I moved that enforcement into a unique constraint in the database, where a conflicting insert can be rejected rather than relying on the earlier check. I haven't benchmarked the change yet, so I can't say what it costs in performance.

The rewrite keeps the concurrent checks, inserts, constraint, and absence of benchmarks, while rebuilding the explanation without the repeated words or staged turn. The added depth comes from explaining the race and the constraint, not inventing tests or an incident. A rewrite need not match the number or order of sentences in its source.

## Developing a mechanism

Suppose the notes say:

> retry after timeout. server might already have processed request. use idempotency key.

A technically accurate explanation can still sound like an inserted lesson:

> An idempotency key is an identifier used to associate multiple requests with a single logical operation. It allows a server to recognize duplicate attempts and return a previously stored result. The client must reuse the identifier when retrying the operation.

An explanation integrated into the discussion could read:

> Suppose we're sending a request to create an order and the server creates it, but the response never reaches us. From our side, all we see is a timeout, so we don't know whether retrying will recover from a failed request or create a second order.
>
> We can give the request an idempotency key: a value the server uses to recognize another attempt at the same operation. If the server stores the result against that key, we can send the request again with the same value and get the original result back instead of creating another order. We need to keep the key when we retry, though; sending a new one would make this look like a new operation.

The second version explains the mechanism through a particular question and keeps the reader inside that explanation. The order is explicitly hypothetical, and the server's behavior is conditional on storing and recognizing the key. There is no invented personal experience. The improvement is in the explanatory approach, not merely the use of “we” or contractions. Do not turn this general mechanism into a particular API's guarantee without checking its documentation.

## Sentence rhythm

Notes:

> API stores times in UTC. report groups by local day. convert before grouping. records near midnight otherwise on wrong day.

Clipped prose to avoid:

> The API stores UTC. The report uses local days. Midnight causes problems. Convert first. Then group.

Connected explanation:

> Although the API stores timestamps in UTC, the report groups records by the user's local calendar day, so the timezone conversion needs to happen before grouping. A record just after midnight UTC can still belong to the previous day for a user west of UTC; grouping it first would put it in the wrong daily total, even if we later displayed the timestamp in local time.

The longer sentence carries a causal relationship, not extra ornament. Do not force every paragraph to this length or make technical simplifications that are false for the case at hand.

## Explanation is not diagnosis

Notes:

> worker RSS grows after repeated image requests. haven't profiled. maybe retained buffers.

A grounded passage:

> After repeated image requests, more of the worker's memory is resident in RAM; that's the increase I'm seeing in RSS. Retained buffers might account for it, but I haven't profiled the worker yet, so I can't tell which allocations are responsible. I'd need that information before calling this a buffer-retention bug.

This explains the measurement and its evidentiary limit without inventing a profiler result, a precise growth rate, or a fix. Match the audience: readers who already know RSS may not need that definition.

## Headings and images

Keep an explanation of the timeout, retry, and idempotency mechanism together under a heading such as “Request retries.” Each step does not need its own heading. Insert an image request only if it helps, for example:

`[Insert a sequence diagram showing a completed request, a lost response, and a retry using the same idempotency key.]`

A narrow edit may need neither headings nor images. The skill supports full blogs and scoped passages without imposing an article template.

## Reference inspiration

The author's independently written OpenWrt build post also provides useful reasoning habits: it explains package removal through the device's access-point role, follows a storage limitation through to configuration loss and workarounds, and names the practical consequence of omitting DHCP. It delegates routine dependency setup to documentation while explaining the environment-specific obstacle. Carry those habits into other topics without copying its dense paragraphs, step headings, rough wording, or historical technical recommendations.

- [Cliffle's Rust essay](https://cliffle.com/blog/prefer-rust/): a scoped position, concrete engineering criteria, exceptions, and objections; useful reasoning supported by personal experience.
- [Fred's Nest Hub investigation](https://fredericb.info/2022/06/breaking-secure-boot-on-google-nest-hub-2nd-gen-to-run-ubuntu.html): observations and evidence connected to subsequent decisions, with personality in the practical details.
- [Fred's Philips TV investigation](https://fredericb.info/2014/11/exploitation-of-philips-smart-tv.html): specific motivation followed by mechanisms and incremental progress.

These observations describe a limited sample and broad techniques, not an author's complete style. The user's facts, preferences, and register govern the output.
