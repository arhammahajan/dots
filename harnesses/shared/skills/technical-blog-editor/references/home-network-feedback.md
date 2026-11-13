# Calibration

The user writes most of the substance, deliberately keeping initial drafts brief, often late at night while sleepy. Develop those compressed ideas into a blog with depth; substantial changes in wording and length are expected. The intended result is technically substantial prose that feels like talking with someone. This guide draws broad editorial lessons from a small sample; it is not a claim about everything either author writes.

## Source observations

- [Cliffle: Prefer Rust to C/C++ for new code](https://cliffle.com/blog/prefer-rust/): a clear position, explicit decision criteria, exceptions, and anticipated objections. Personal experience supports the argument. Brief asides and sentence-length changes give dense material breathing room. Borrow the practice of making reasoning and scope visible; do not borrow the combative stance or personal history.
- [Fred: Breaking Secure Boot on Google Nest Hub](https://fredericb.info/2022/06/breaking-secure-boot-on-google-nest-hub-2nd-gen-to-run-ubuntu.html): a stated goal followed by observations, setbacks, and successive experiments. Evidence appears close to its interpretation. Small reactions make the process feel inhabited without replacing the technical work. Borrow the connections between evidence and the next decision, rather than forcing an investigation narrative onto every topic.
- [Fred: Exploitation of Philips Smart TV](https://fredericb.info/2014/11/exploitation-of-philips-smart-tv.html): a concrete personal motivation leads quickly into technical evidence. Sections develop capabilities step by step, distinguishing a hypothesis from later progress. Borrow the efficient movement from motivation to mechanism; do not imitate incidental grammar or punctuation.

The shared editorial target is a reader who understands both what happened and why it mattered. Approachability does not require less detail or constant jokes. The user's facts and viewpoint take precedence over these observations; the examples guide how to express them.

## Rhythm calibration

The user still found the prose too clipped after the previous feedback was incorporated. Avoid interpreting “plain,” “direct,” and “matter-of-fact” as a request for short, isolated statements. The heading rules and body rhythm are separate requirements.

These passages use facts from the home-network draft. The first illustrates the unwanted rhythm; the second demonstrates how the same material can form a connected explanation.

Clipped:

> The Nokia had 512 MB of RAM. It had 256 MB of flash. It was more capable than the TP-Link. The firmware was locked down. I couldn't change DNS. I couldn't assign static IPs. I looked for an OpenWrt port. Drivers were missing.

Connected:

> With 512 MB of RAM and 256 MB of flash, the Nokia ONT was considerably more capable than my old TP-Link, but Airtel's firmware left me with very little control over it. I could change the Wi-Fi name and password, yet couldn't do things as basic as choosing an upstream DNS server or assigning static IP addresses to my devices. That was a frustrating limitation on hardware that otherwise had much more to offer.
>
> I looked into running OpenWrt on the ONT and found that people had already done a fair amount of work on its Econet SoC. At the time, though, missing drivers meant there wasn't a port that booted into a usable state, so I couldn't simply replace the firmware and configure the device the way I wanted.

The revision uses comparison, contrast, and qualification inside sentences, allowing the explanation to continue across the paragraph. It preserves a reaction supported by the draft without adding an incident or a dramatic reveal. There is no sentence-length target: a short sentence is fine when the thought needs one. Do not copy these sentence patterns into every paragraph or expand an already clear passage simply to make it longer.

## Feedback from a real draft and rewrite

The author supplied a home-network draft and an AI rewrite for comparison. These are observations from that pair, not a claim about how an unspecified model always behaves. The original had about 1,399 words and three section headings; the rewrite had about 2,310 words and thirteen. Length increased while important source material disappeared. More prose alone did not mean better development.

### What worked

- The title “Home network with OpenWrt” and hardware names as headings were plain and specific. The remaining structure problem was the number and placement of headings, not movie-like heading wording in this particular sample.
- The rewrite made the old router's memory limits and the contrast with the restricted ISP device easier to follow. Preserve that kind of clarification, without asserting undocumented implementation details.
- Image prompts were useful: an ONT photo, a topology diagram, and a telnet screenshot with actual identifiers redacted. Keep the specificity and inline placement. A topology diagram should request confirmation of unspecified links rather than invent the wiring.
- A compact list made the VLAN policies easier to scan. Parallel configuration facts benefit from a list; the surrounding discussion can remain prose. Keep unspecified membership and access rights out of the list.
- The rewrite retained that the NAND-wear reasoning came from ChatGPT, rather than presenting it as an independently measured finding. That attribution is valuable, although the rewrite also added unsupported reasons for disabling logs.
- More developed paragraphs and natural contractions were useful. Retain those improvements while correcting factual and tonal drift.

### Coverage failure

The draft said the Pi ran Vaultwarden, a WebDAV server for local synchronization, and Homebridge, with Bitwarden clients working across the author's devices. It also identified the Pi's memory as a constraint on the author's setup. The rewrite mentioned the Pi and Vaultwarden but dropped the dedicated account of its services, synchronization, client experience, and resource constraint. Mentioning Homebridge in a different paragraph did not preserve this coverage.

A fuller rewrite must account for those distinct facts. If the sentence about exposing devices “to homebridge” is ambiguous, ask which destination the author means rather than silently changing it. Keep memory limitations scoped to the author's experience instead of asserting a universal inability to run containers.

### Unsupported personal detail

The rewrite supplied details absent from the draft:

- The router was put in a drawer; particular wireless, firewall, and DHCP settings were redone after reboot.
- “A couple times” became exactly two soft bricks and two recoveries.
- “The ONT did not work” became “no link and no internet,” which asserts a particular failure symptom.
- Too many filter lists led to a power cycle; the author never read the logs; disabling logs cost nothing.
- The main VLAN contained particular devices and had full access; guest was the least-used VLAN; Chromecast and a NAS appeared as if part of the actual setup.
- The failover had never carried traffic, and the author planned to rebuild and test it in a particular way.
- The author read diffs and verified firewall and route changes on the device. The original advised double-checking; it did not describe those concrete habits.

These details are plausible, but a rewrite cannot report them as experience. “The whole process took over two weeks” also became time spent finding commands and gaining access. That changes the scope of a supplied duration.

General lessons: do not turn approximate quantities into exact counts, recommendations into completed actions, possible motives into stated reasons, or general explanations into facts about the user's devices.

### Technical authority without evidence

The rewrite explained CPU pressure by asserting that every DNS query checks every rule in every list, then generalized that router CPUs cannot handle tens of thousands of rules. The source gave a symptom, not that implementation or a capacity limit. AdGuard's DNS filtering library has indexed host-rule lookups; the asserted exhaustive scan is not a sound explanation. See the [primary implementation](https://github.com/AdguardTeam/urlfilter/blob/master/dnsengine.go).

The source's hardware-NAT recommendation also needs checking before being expanded into a Tailscale performance requirement. [Tailscale's performance guidance](https://tailscale.com/docs/reference/best-practices/performance) discusses UDP transport-layer offloads for supported Linux subnet routers and exit nodes. That is not evidence that OpenWrt hardware NAT acceleration handles all Tailscale traffic. Preserve any reported improvement separately from the unverified explanation.

Likewise, do not strengthen a reported discovery setup into claims that a specific Chromecast exists, multicast is the only discovery mechanism, or a repeater guarantees isolation. Ask for the actual setup or verify the general mechanism where needed. The editorial lesson is to distinguish source observations from added causes and guarantees, not to memorize networking-specific rules.

### Voice drift

The draft already had a human voice: enthusiasm about porting OpenWrt, frustration at locked DNS settings, a joke about buying hardware despite a working network, and “don't ask me how I know” about blocklists. Preserve that personality without increasing its frequency.

The rewrite added a different persona through recurring metaphors, dismissive judgments, and polished closing lines: useful parts were “welded shut,” an ONT was “a stranger to the OLT,” a backup path “quietly rots,” and the repeater's adequacy became another joke at the old router's expense. The opening added stability and an end to tinkering that the source did not establish. These additions create a seasoned, cynical narrator and a tidier story than the author's account supports.

Do not solve this by deleting all opinion, humor, or first person. Keep the author's particular reactions and use the added space to explain the work. Not every paragraph needs a verdict, joke, metaphor, or lesson.

### Section structure

Thirteen headings fragmented this overview. The ONT's limitations, OpenWrt research, and replacement belong in one discussion. Hardware, router services, the switch, the Pi, the repeater, and the short failover note can be grouped into a few substantial sections. A single short paragraph about multi-WAN does not need a heading merely because it introduces a feature.

One possible organization for this material is:

- TP-Link WR840N
- Nokia ONT
- Current setup
- Configuration and maintenance

This is a sample grouping, not a required count or template. A separate substantial discussion of VLAN policy could earn its own section. Related observations and results should stay together even when they each have enough vocabulary for a heading.

### Grounded development example

Source material:

> I primarily run tailscale and adguard on this device. While I initially was running vaultwarden on it too, i have since migrated it to my raspberry pi zero 2w instead.

> The raspberry pi is running vaultwarden which works flawlessly with bitwarden across all my devices, a webdav server for some local file sync, and homebridge for exposing (unsupported) iot devices to homebridge. The 512mb ram on the pi prevents me from running docker or more services.

A grounded passage:

> The M30 runs Tailscale and AdGuard. Vaultwarden used to run there too, but I've moved it to the Pi Zero 2W. That leaves the router handling the network services, with the Pi also running a WebDAV server for local file sync and Homebridge for the IoT devices.
>
> Vaultwarden has worked well with the Bitwarden clients across my devices. WebDAV handles some local synchronization, so the Pi has a few useful jobs despite its limited memory. With 512 MB of RAM, I've kept this setup small and haven't added Docker or more services.

This preserves distinct roles and the author's experience without adding a reason for the migration, a container benchmark, or a fictional incident. The ambiguous Homebridge destination still warrants a short author question outside the prose. More supplied details could support further expansion; unsupported details should not be added to reach a target length.

### Check the next rewrite

Use this case to inspect behavior, not to impose its outline on unrelated posts. Check substantive coverage, support for added personal claims, scope of technical explanations, the author's original register, and whether small sections can be merged. A structurally valid skill file does not establish that the next model output passes these checks.
