#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { Octokit } from '@octokit/rest';

// GitHub client
const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN,
});

const REPO_OWNER = process.env.GITHUB_OWNER || 'Shivam-Bhardwaj';
const REPO_NAME = process.env.GITHUB_REPO || 'S3M2P';

// MCP Server
const server = new Server(
  {
    name: 'github-automation',
    version: '0.1.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Tool: Read issue with all metadata
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'github_issue_read',
        description: 'Read a GitHub issue with all metadata including comments, labels, and state',
        inputSchema: {
          type: 'object',
          properties: {
            issue_number: {
              type: 'number',
              description: 'The issue number to read',
            },
          },
          required: ['issue_number'],
        },
      },
      {
        name: 'github_issue_comment',
        description: 'Post a comment to a GitHub issue',
        inputSchema: {
          type: 'object',
          properties: {
            issue_number: {
              type: 'number',
              description: 'The issue number to comment on',
            },
            body: {
              type: 'string',
              description: 'The comment body (markdown supported)',
            },
          },
          required: ['issue_number', 'body'],
        },
      },
      {
        name: 'github_issue_comments',
        description: 'List all comments on a GitHub issue',
        inputSchema: {
          type: 'object',
          properties: {
            issue_number: {
              type: 'number',
              description: 'The issue number',
            },
            since: {
              type: 'string',
              description: 'Only show comments updated after this ISO 8601 timestamp',
            },
          },
          required: ['issue_number'],
        },
      },
      {
        name: 'github_pr_create',
        description: 'Create a pull request',
        inputSchema: {
          type: 'object',
          properties: {
            issue_number: {
              type: 'number',
              description: 'The issue number this PR addresses',
            },
            title: {
              type: 'string',
              description: 'PR title',
            },
            body: {
              type: 'string',
              description: 'PR description (markdown supported)',
            },
            head: {
              type: 'string',
              description: 'The branch containing changes',
            },
            base: {
              type: 'string',
              description: 'The branch to merge into (default: main)',
            },
          },
          required: ['title', 'body', 'head'],
        },
      },
      {
        name: 'github_ci_status',
        description: 'Check CI status for a branch or commit',
        inputSchema: {
          type: 'object',
          properties: {
            ref: {
              type: 'string',
              description: 'Branch name or commit SHA',
            },
          },
          required: ['ref'],
        },
      },
    ],
  };
});

// Tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case 'github_issue_read': {
        const { issue_number } = args as { issue_number: number };

        const { data: issue } = await octokit.issues.get({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          issue_number,
        });

        const { data: comments } = await octokit.issues.listComments({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          issue_number,
        });

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  number: issue.number,
                  title: issue.title,
                  body: issue.body,
                  state: issue.state,
                  labels: issue.labels.map((l: any) => l.name),
                  created_at: issue.created_at,
                  updated_at: issue.updated_at,
                  user: issue.user?.login,
                  comments_count: issue.comments,
                  comments: comments.map((c) => ({
                    id: c.id,
                    user: c.user?.login,
                    body: c.body,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                  })),
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case 'github_issue_comment': {
        const { issue_number, body } = args as {
          issue_number: number;
          body: string;
        };

        const { data: comment } = await octokit.issues.createComment({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          issue_number,
          body,
        });

        return {
          content: [
            {
              type: 'text',
              text: `Comment posted successfully! URL: ${comment.html_url}`,
            },
          ],
        };
      }

      case 'github_issue_comments': {
        const { issue_number, since } = args as {
          issue_number: number;
          since?: string;
        };

        const { data: comments } = await octokit.issues.listComments({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          issue_number,
          since,
        });

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                comments.map((c) => ({
                  id: c.id,
                  user: c.user?.login,
                  body: c.body,
                  created_at: c.created_at,
                  updated_at: c.updated_at,
                })),
                null,
                2
              ),
            },
          ],
        };
      }

      case 'github_pr_create': {
        const { issue_number, title, body, head, base = 'main' } = args as {
          issue_number?: number;
          title: string;
          body: string;
          head: string;
          base?: string;
        };

        // Add issue reference if provided
        const prBody = issue_number
          ? `${body}\n\nCloses #${issue_number}`
          : body;

        const { data: pr } = await octokit.pulls.create({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          title,
          body: prBody,
          head,
          base,
        });

        return {
          content: [
            {
              type: 'text',
              text: `PR created successfully!\nNumber: #${pr.number}\nURL: ${pr.html_url}`,
            },
          ],
        };
      }

      case 'github_ci_status': {
        const { ref } = args as { ref: string };

        const { data: checks } = await octokit.checks.listForRef({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          ref,
        });

        const { data: statuses } = await octokit.repos.getCombinedStatusForRef({
          owner: REPO_OWNER,
          repo: REPO_NAME,
          ref,
        });

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  combined_state: statuses.state,
                  total_count: statuses.total_count,
                  checks: checks.check_runs.map((c) => ({
                    name: c.name,
                    status: c.status,
                    conclusion: c.conclusion,
                  })),
                  statuses: statuses.statuses.map((s) => ({
                    context: s.context,
                    state: s.state,
                    description: s.description,
                  })),
                },
                null,
                2
              ),
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error: any) {
    return {
      content: [
        {
          type: 'text',
          text: `Error: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);

  console.error('GitHub MCP server running on stdio');
}

main().catch((error) => {
  console.error('Server error:', error);
  process.exit(1);
});
