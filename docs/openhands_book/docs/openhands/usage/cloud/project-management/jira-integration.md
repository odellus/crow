# Jira Cloud Integration

> Complete guide for setting up Jira Cloud integration with OpenHands Cloud, including service account creation, API token generation, webhook configuration, and workspace integration setup.

# Jira Cloud Integration

## Platform Configuration

### Step 1: Create Service Account

1. **Navigate to User Management**
   * Go to [Atlassian Admin](https://admin.atlassian.com/)
   * Select your organization
   * Go to **Directory** > **Users**

2. **Create OpenHands Service Account**
   * Click **Service accounts**
   * Click **Create a service account**
   * Name: `OpenHands Agent`
   * Click **Next**
   * Select **User** role for Jira app
   * Click **Create**

### Step 2: Generate API Token

1. **Access Service Account Configuration**
   * Locate the created service account from above step and click on it
   * Click **Create API token**
   * Set the expiry to 365 days (maximum allowed value)
   * Click **Next**
   * In **Select token scopes** screen, filter by following values
     * App: Jira
     * Scope type: Classic
     * Scope actions: Write, Read
   * Select `read:me`, `read:jira-work`, and `write:jira-work` scopes
   * Click **Next**
   * Review and create API token
   * **Important**: Copy and securely store the token immediately

### Step 3: Configure Webhook

1. **Navigate to Webhook Settings**
   * Go to **Jira Settings** > **System** > **WebHooks**
   * Click **Create a WebHook**

2. **Configure Webhook**
   * **Name**: `OpenHands Cloud Integration`
   * **Status**: Enabled
   * **URL**: `https://app.all-hands.dev/integration/jira/events`
   * **Issue related events**: Select the following:
     * Issue updated
     * Comment created
   * **JQL Filter**: Leave empty (or customize as needed)
   * Click **Create**
   * **Important**: Copy and store the webhook secret securely (you'll need this for workspace integration)

***

## Workspace Integration

### Step 1: Log in to OpenHands Cloud

1. **Navigate and Authenticate**
   * Go to [OpenHands Cloud](https://app.all-hands.dev/)
   * Sign in with your Git provider (GitHub, GitLab, or BitBucket)
   * **Important:** Make sure you're signing in with the same Git provider account that contains the repositories you want the OpenHands agent to work on.

### Step 2: Configure Jira Integration

1. **Access Integration Settings**
   * Navigate to **Settings** > **Integrations**
   * Locate **Jira Cloud** section

2. **Configure Workspace**
   * Click **Configure** button
   * Enter your workspace name and click **Connect**
   * **Important:** Make sure you enter the full workspace name, eg: **yourcompany.atlassian.net**
     * If no integration exists, you'll be prompted to enter additional credentials required for the workspace integration:
       * **Webhook Secret**: The webhook secret from Step 3 above
       * **Service Account Email**: The service account email from Step 1 above
       * **Service Account API Key**: The API token from Step 2 above
       * Ensure **Active** toggle is enabled

<Note>
  Workspace name is the host name when accessing a resource in Jira Cloud.

  Eg: [https://all-hands.atlassian.net/browse/OH-55](https://all-hands.atlassian.net/browse/OH-55)

  Here the workspace name is **all-hands**.
</Note>

3. **Complete OAuth Flow**
   * You'll be redirected to Jira Cloud to complete OAuth verification
   * Grant the necessary permissions to verify your workspace access.
   * If successful, you will be redirected back to the **Integrations** settings in the OpenHands Cloud UI

### Managing Your Integration

**Edit Configuration:**

* Click the **Edit** button next to your configured platform
* Update any necessary credentials or settings
* Click **Update** to apply changes
* You will need to repeat the OAuth flow as before
* **Important:** Only the original user who created the integration can see the edit view

**Unlink Workspace:**

* In the edit view, click **Unlink** next to the workspace name
* This will deactivate your workspace link
* **Important:** If the original user who configured the integration chooses to unlink their integration, any users currently linked to that workspace integration will also be unlinked, and the workspace integration will be deactivated. The integration can only be reactivated by the original user.

### Screenshots

<AccordionGroup>
  <Accordion title="Workspace link flow">
        <img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=bc4fa18e6a3e9cc15cd62f9c7eabc140" alt="workspace-link.png" data-og-width="402" width="402" data-og-height="430" height="430" data-path="openhands/static/img/jira-user-link.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2a1dd32146c1d9168ee0e9cced95c221 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=834c40fac316ca8001b4ddfee634eeda 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=6728434d30e0e9a68fd9dc4e7e413360 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=b465b4b86ec3c6368eeb89786302047b 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=32d211ec3924148a2e620b234e088a29 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-link.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d2e3571d1274cf99080af9c83391abb4 2500w" />
  </Accordion>

  <Accordion title="Workspace Configure flow">
        <img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=5fdb2c28d9fdd4bf4290865451c38738" alt="workspace-link.png" data-og-width="402" width="402" data-og-height="742" height="742" data-path="openhands/static/img/jira-admin-configure.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=0c034d93857fe70fb339d50342f5bd69 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=bca918d0e3a37c4a81428191aa88275f 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=c09101a7cb393a7270cbf69993306313 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=f920278659c9dd8e58ab162fad86ebfd 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=d1b97e503c1ce0a8fa096a486b0d721c 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-configure.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=7015dc41bb84f32aa592c9226ab96593 2500w" />
  </Accordion>

  <Accordion title="Edit view as a user">
        <img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=6ca04d21a5c994684271ccf052ea265e" alt="workspace-link.png" data-og-width="402" width="402" data-og-height="385" height="385" data-path="openhands/static/img/jira-user-unlink.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=f2a93fad91fcf97559c23a48aed37c1d 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=b1aa4644bbc80f6ed5ce29f4b7d14a7c 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=6eb7f6072bf9765f58a93d42d8a4b716 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=aee8465ecfbbeaf0c3dbf0d2b622fb3f 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=737e9173f2ebe4b18563169fb537786d 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-user-unlink.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=2f4bf0f691b4ec8431ebf7fe3e623adc 2500w" />
  </Accordion>

  <Accordion title="Edit view as the workspace creator">
        <img src="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=4aa37f571a5edc886c87db6bb09388da" alt="workspace-link.png" data-og-width="402" width="402" data-og-height="750" height="750" data-path="openhands/static/img/jira-admin-edit.png" data-optimize="true" data-opv="3" srcset="https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=280&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=8044bf91834c9a36c9ede64081a51938 280w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=560&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=66aa27171a1b1fd354bc0d00397ab9c7 560w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=840&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=944c3079c9d7a838b9defa59a84fa8e5 840w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=1100&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=ae8af8b444ac534a410aeb8b1926f37d 1100w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=1650&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=aaf81acbcf61af8785151650fa0bacf5 1650w, https://mintcdn.com/allhandsai/iROoLZU8-F_m1dYO/openhands/static/img/jira-admin-edit.png?w=2500&fit=max&auto=format&n=iROoLZU8-F_m1dYO&q=85&s=905b92dd667ad4089492acdaf3a2f8fd 2500w" />
  </Accordion>
</AccordionGroup>


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt