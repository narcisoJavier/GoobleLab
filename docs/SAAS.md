# GoobleLab Free and Pro direction

GoobleLab's browser app is the primary interface; the editable Colab notebook stays the core output. Charge for configuration convenience and curated workflows while preserving direct use of the MIT-licensed GoobleLab notebook and runner. MiniMax H3 model rights remain governed by its separate license.

## Proposed offer

| Offer | Scope |
| --- | --- |
| Free | Basic MiniMax H3 text-to-video builder: prompt and name, FL2VA Turbo 8, 1344×768, requested 5 seconds, seed `12345`, editable notebook download and local projects. |
| Pro | Advanced browser customization: image/reference workflows, all five presets, resolution, duration, seed and compatible custom LoRA controls. Future value could include premium curated workflows and richer reusable profiles. |

The current **Pro preview** toggle demonstrates these controls without payment. It is not subscription enforcement. Pricing is undecided; accounts, checkout, recurring billing and cloud sync are not implemented. Exported notebooks remain editable, so experienced users can configure the same underlying model settings themselves.

Local project saving already works in both modes. Future paid profiles should provide additional convenience rather than presenting existing local saving as an unavailable cloud feature. Any later hosted inference would need its own infrastructure, rights and compute economics.

## Before taking payments

Use authenticated accounts, a payment provider and server-verified subscription state. Verify signed payment webhooks and handle cancellations, refunds, failures and access expiry. A client-side plan variable cannot protect a paid service; keep anything requiring entitlement behind server authorization.

Publish accurate pricing, service and privacy terms, retention/deletion rules, and support arrangements. Review the OAuth consent setup and intended public distribution; test live authorization, least-privilege Drive uploads and failure recovery. Validate the supported notebook workflows on actual Colab hardware and describe compute costs separately from a GoobleLab subscription.

## Colab and model terms

The intended product prepares files and settings for each user's own Colab account. [Colab's additional terms](https://research.google.com/colaboratory/tos_v6.html) prohibit account sharing and selling, renting or providing third parties direct or indirect access to Colab. Do not package access to operator-controlled Colab runtimes as paid generation credits. This architecture still needs review against the service's exact behavior and current terms before launch.

The [MiniMax H3 Community License](https://huggingface.co/MiniMaxAI/MiniMax-H3/blob/main/LICENSE), reviewed November 7, 2026, excludes the EU, UK, Republic of Korea and US from its default territory. It requires prominent MiniMax H3 branding in commercial interfaces, and prior written authorization when yearly commercial revenue exceeds US$20 million. It also addresses license notices, downstream use restrictions and safeguards.

Determine which provisions apply to this notebook builder and its intended customers, hosting and distribution before a public paid launch. Do not assume worldwide SaaS permission or that the repository's MIT license replaces the model license. These are launch questions for qualified review, not a legal clearance from this proposal.
