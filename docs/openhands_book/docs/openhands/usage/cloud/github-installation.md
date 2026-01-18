# GitHub Integration

> This guide walks you through the process of installing OpenHands Cloud for your GitHub repositories. Once set up, it will allow OpenHands to work with your GitHub repository through the Cloud UI or straight from GitHub!

## Prerequisites

* Signed in to [OpenHands Cloud](https://app.all-hands.dev) with [a GitHub account](/openhands/usage/cloud/openhands-cloud).

## Adding GitHub Repository Access

You can grant OpenHands access to specific GitHub repositories:

1. Click on `+ Add GitHub Repos` in the repository selection dropdown.
2. Select your organization and choose the specific repositories to grant OpenHands access to.

<Accordion title="OpenHands permissions">
  * OpenHands requests short-lived tokens (8-hour expiration) with these permissions:
    * Actions: Read and write
    * Commit statuses: Read and write
    * Contents: Read and write
    * Issues: Read and write
    * Metadata: Read-only
    * Pull requests: Read and write
    * Webhooks: Read and write
    * Workflows: Read and write
  * Repository access for a user is granted based on:
    * Permission granted for the repository
    * User's GitHub permissions (owner/collaborator)
</Accordion>

3. Click `Install & Authorize`.

## Modifying Repository Access

You can modify GitHub repository access at any time by:

* Selecting `+ Add GitHub Repos` in the repository selection dropdown or
* Visiting the `Settings > Integrations` page and selecting `Configure GitHub Repositories`

## Working With GitHub Repos in Openhands Cloud

Once you've granted GitHub repository access, you can start working with your GitHub repository. Use the
`Open Repository` section to select the appropriate repository and branch you'd like OpenHands to work on. Then click
on `Launch` to start the conversation!

<img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=e6b11dcf83062ccee3a22d107440c976" alt="Connect Repo" data-og-width="344" width="344" data-og-height="269" height="269" data-path="openhands/static/img/connect-repo.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=0cc85cb018c8f777b12e61a6dc5ae19c 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=f17fb0d0921687f84ecf70af8397f66e 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=48daba417c153e0de6bde29dd624fad7 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=232731f674c4b56709577803909357de 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=8040c688490febdd6c9d7df0b7bc0fcf 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/connect-repo.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=3810134f59fa44661ff3e6bc4772cff0 2500w" />

## Working on GitHub Issues and Pull Requests Using Openhands

To allow OpenHands to work directly from GitHub directly, you must
[give OpenHands access to your repository](/openhands/usage/cloud/github-installation#modifying-repository-access). Once access is
given, you can use OpenHands by labeling the issue or by tagging `@openhands`.

### Working with Issues

On your repository, label an issue with `openhands` or add a message starting with `@openhands`. OpenHands will:

1. Comment on the issue to let you know it is working on it.
   * You can click on the link to track the progress on OpenHands Cloud.
2. Open a pull request if it determines that the issue has been successfully resolved.
3. Comment on the issue with a summary of the performed tasks and a link to the PR.

### Working with Pull Requests

To get OpenHands to work on pull requests, mention `@openhands` in the comments to:

* Ask questions
* Request updates
* Get code explanations

<Note>
  The `@openhands` mention functionality in pull requests only works if the pull request is both
  *to* and *from* a repository that you have added through the interface. This is because OpenHands needs appropriate
  permissions to access both repositories.
</Note>

## Next Steps

* [Learn about the Cloud UI](/openhands/usage/cloud/cloud-ui).
* [Use the Cloud API](/openhands/usage/cloud/cloud-api) to programmatically interact with OpenHands.


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt