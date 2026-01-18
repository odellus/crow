# Slack Integration

> This guide walks you through installing the OpenHands Slack app.

<iframe className="w-full aspect-video" src="https://www.youtube.com/embed/hbloGmfZsJ4" title="OpenHands Slack Integration Tutorial" frameBorder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowFullScreen />

<Info>
  OpenHands utilizes a large language model (LLM), which may generate responses that are inaccurate or incomplete.
  While we strive for accuracy, OpenHands' outputs are not guaranteed to be correct, and we encourage users to
  validate critical information independently.
</Info>

## Prerequisites

* Access to OpenHands Cloud.

## Installation Steps

<AccordionGroup>
  <Accordion title="Install Slack App (only for Slack admins/owners)">
    **This step is for Slack admins/owners**

    1. Make sure you have permissions to install Apps to your workspace.
    2. Click the button below to install OpenHands Slack App <a target="_blank" href="https://slack.com/oauth/v2/authorize?client_id=7477886716822.8729519890534&scope=app_mentions:read,channels:history,chat:write,groups:history,im:history,mpim:history,users:read&user_scope="><img alt="Add to Slack" height="40" width="139" src="https://platform.slack-edge.com/img/add_to_slack.png" srcSet="https://platform.slack-edge.com/img/add_to_slack.png 1x, https://platform.slack-edge.com/img/add_to_slack@2x.png 2x" /></a>
    3. In the top right corner, select the workspace to install the OpenHands Slack app.
    4. Review permissions and click allow.
  </Accordion>

  <Accordion title="Authorize Slack App (for all Slack workspace members)">
    **Make sure your Slack workspace admin/owner has installed OpenHands Slack App first.**

    Every user in the Slack workspace (including admins/owners) must link their OpenHands Cloud account to the OpenHands Slack App. To do this:

    1. Visit the [Settings > Integrations](https://app.all-hands.dev/settings/integrations) page in OpenHands Cloud.
    2. Click `Install OpenHands Slack App`.
    3. In the top right corner, select the workspace to install the OpenHands Slack app.
    4. Review permissions and click allow.

    Depending on the workspace settings, you may need approval from your Slack admin to authorize the Slack App.
  </Accordion>
</AccordionGroup>

## Working With the Slack App

To start a new conversation, you can mention `@openhands` in a new message or a thread inside any Slack channel.

Once a conversation is started, all thread messages underneath it will be follow-up messages to OpenHands.

To send follow-up messages for the same conversation, mention `@openhands` in a thread reply to the original message.
You must be the user who started the conversation.

## Example conversation

### Start a new conversation, and select repo

Conversation is started by mentioning `@openhands`.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=5f23b3b96e0875a2d178b71af2c3e359" alt="slack-create-conversation.png" data-og-width="1308" width="1308" data-og-height="496" height="496" data-path="openhands/static/img/slack-create-conversation.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=163290930905b3a8a6c63ee3c5a0b303 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2f456d897ac97cb3ad5f52de5655fc05 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=3bbbda059a451c657afe86fb3337eefa 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=4cf5a78c77a1a9719be330e1bcaaf089 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2d4bde6bd2fbd4a72d91f03324a4a32c 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-create-conversation.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9f23d45bd8bf893513b498425f334952 2500w" />

### See agent response and send follow up messages

Initial request is followed up by mentioning `@openhands` in a thread reply.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=33c624490e6ea5ad72e1838eccbbd6e5" alt="slack-results-and-follow-up.png" data-og-width="1604" width="1604" data-og-height="1558" height="1558" data-path="openhands/static/img/slack-results-and-follow-up.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=c8a985a98d62285950dda724db9524fe 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2f81eb26e457c04ffc3201fae00e4595 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=12231f7c80e90b7c83ebc82f7acb8b32 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=e48365b5a448e7a1cff50e45d760159b 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=20ef214ee42b337e5af60f4789a26cb4 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-results-and-follow-up.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=b21be4076790443fd7106f41c8156806 2500w" />

## Pro tip

You can mention a repo name when starting a new conversation in the following formats

1. "My-Repo" repo (e.g `@openhands in the openhands repo ...`)
2. "OpenHands/OpenHands" (e.g `@openhands in OpenHands/OpenHands ...`)

The repo match is case insensitive. If a repo name match is made, it will kick off the conversation.
If the repo name partially matches against multiple repos, you'll be asked to select a repo from the filtered list.

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=140a1c31cdefd5ef9e0ba79a1228e154" alt="slack-pro-tip.png" data-og-width="998" width="998" data-og-height="634" height="634" data-path="openhands/static/img/slack-pro-tip.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=3bbee578033604aa17baf5b2a9117db1 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=1b41dbd066d2e86cafafa4b094140155 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=9dba50f90915149e8914c1b4afac89ca 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=fc59cbe6fe9cbf5fa93f354aa2f8402a 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=b3a3df104143243df71a8ac90fbb0bda 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/slack-pro-tip.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=106b1218e7163cb7056a91deba9bb468 2500w" />


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt