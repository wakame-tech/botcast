#!/usr/bin/env node
/**
 * MCP Server generated from OpenAPI spec for botcast-cms v0.0.0
 * Generated on: 2026-05-04T04:34:00.031Z
 */

// Load environment variables from .env file
import dotenv from 'dotenv';
dotenv.config();

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  type Tool,
  type CallToolResult,
  type CallToolRequest
} from "@modelcontextprotocol/sdk/types.js";

import { z, ZodError } from 'zod';
import { jsonSchemaToZod } from 'json-schema-to-zod';
import axios, { type AxiosRequestConfig, type AxiosError } from 'axios';

/**
 * Type definition for JSON objects
 */
type JsonObject = Record<string, any>;

/**
 * Interface for MCP Tool Definition
 */
interface McpToolDefinition {
    name: string;
    description: string;
    inputSchema: any;
    method: string;
    pathTemplate: string;
    executionParameters: { name: string, in: string }[];
    requestBodyContentType?: string;
    securityRequirements: any[];
}

/**
 * Server configuration
 */
export const SERVER_NAME = "botcast-cms";
export const SERVER_VERSION = "0.0.0";
export const API_BASE_URL = "http://localhost:3002";

/**
 * MCP Server instance
 */
const server = new Server(
    { name: SERVER_NAME, version: SERVER_VERSION },
    { capabilities: { tools: {} } }
);

/**
 * Map of tool definitions by name
 */
const toolDefinitionMap: Map<string, McpToolDefinition> = new Map([

  ["CanvasApi_listEdges", {
    name: "CanvasApi_listEdges",
    description: `Executes GET /canvas/edges`,
    inputSchema: {"type":"object","properties":{"record_id":{"type":"string"}},"required":["record_id"]},
    method: "get",
    pathTemplate: "/canvas/edges",
    executionParameters: [{"name":"record_id","in":"query"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["CanvasApi_createEdge", {
    name: "CanvasApi_createEdge",
    description: `Executes POST /canvas/edges`,
    inputSchema: {"type":"object","properties":{"requestBody":{"type":"object","required":["from","to","label"],"properties":{"from":{"type":"string"},"to":{"type":"string"},"label":{"type":"string"}},"description":"The JSON request body."}},"required":["requestBody"]},
    method: "post",
    pathTemplate: "/canvas/edges",
    executionParameters: [],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["CanvasApi_deleteEdge", {
    name: "CanvasApi_deleteEdge",
    description: `Executes DELETE /canvas/edges/{edgeId}`,
    inputSchema: {"type":"object","properties":{"edgeId":{"type":"string"}},"required":["edgeId"]},
    method: "delete",
    pathTemplate: "/canvas/edges/{edgeId}",
    executionParameters: [{"name":"edgeId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["CanvasApi_query", {
    name: "CanvasApi_query",
    description: `Executes POST /canvas/query`,
    inputSchema: {"type":"object","properties":{"requestBody":{"type":"object","required":["record_id","format"],"properties":{"record_id":{"type":"string","description":"Starting record ID in \"table:id\" format"},"depth":{"type":"number","description":"Graph traversal depth (default: 1)"},"conditions":{"type":"object","unevaluatedProperties":{"type":"object"},"description":"Filter conditions on record fields"},"format":{"type":"string","enum":["json-canvas","markdown"],"description":"Export format"}},"description":"The JSON request body."}},"required":["requestBody"]},
    method: "post",
    pathTemplate: "/canvas/query",
    executionParameters: [],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["CollectionApi_list", {
    name: "CollectionApi_list",
    description: `Executes GET /collections`,
    inputSchema: {"type":"object","properties":{}},
    method: "get",
    pathTemplate: "/collections",
    executionParameters: [],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["CollectionApi_create", {
    name: "CollectionApi_create",
    description: `Executes POST /collections`,
    inputSchema: {"type":"object","properties":{"requestBody":{"type":"object","required":["name","schema"],"properties":{"name":{"type":"string"},"schema":{"type":"object","unevaluatedProperties":{"type":"string"}}},"description":"The JSON request body."}},"required":["requestBody"]},
    method: "post",
    pathTemplate: "/collections",
    executionParameters: [],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["CollectionApi_read", {
    name: "CollectionApi_read",
    description: `Executes GET /collections/{collectionId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"}},"required":["collectionId"]},
    method: "get",
    pathTemplate: "/collections/{collectionId}",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["CollectionApi_update", {
    name: "CollectionApi_update",
    description: `Executes PUT /collections/{collectionId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"requestBody":{"type":"object","required":["name"],"properties":{"name":{"type":"string"}},"description":"The JSON request body."}},"required":["collectionId","requestBody"]},
    method: "put",
    pathTemplate: "/collections/{collectionId}",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["CollectionApi_delete", {
    name: "CollectionApi_delete",
    description: `Executes DELETE /collections/{collectionId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"}},"required":["collectionId"]},
    method: "delete",
    pathTemplate: "/collections/{collectionId}",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["JobApi_list", {
    name: "JobApi_list",
    description: `Executes GET /jobs`,
    inputSchema: {"type":"object","properties":{}},
    method: "get",
    pathTemplate: "/jobs",
    executionParameters: [],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["JobApi_create", {
    name: "JobApi_create",
    description: `Executes POST /jobs`,
    inputSchema: {"type":"object","properties":{"requestBody":{"type":"object","required":["name","params"],"properties":{"name":{"type":"string"},"params":{"type":"object","unevaluatedProperties":{"type":"object"}}},"description":"The JSON request body."}},"required":["requestBody"]},
    method: "post",
    pathTemplate: "/jobs",
    executionParameters: [],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_list", {
    name: "RecordApi_list",
    description: `Executes GET /records/{collectionId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"}},"required":["collectionId"]},
    method: "get",
    pathTemplate: "/records/{collectionId}",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["RecordApi_create", {
    name: "RecordApi_create",
    description: `Executes POST /records/{collectionId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"requestBody":{"type":"object","required":["data"],"properties":{"data":{"type":"object","unevaluatedProperties":{"type":"object"}}},"description":"The JSON request body."}},"required":["collectionId","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_generate", {
    name: "RecordApi_generate",
    description: `Executes POST /records/{collectionId}/generate`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"requestBody":{"type":"object","required":["prompt"],"properties":{"prompt":{"type":"string"}},"description":"The JSON request body."}},"required":["collectionId","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}/generate",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_mergeRecords", {
    name: "RecordApi_mergeRecords",
    description: `Executes POST /records/{collectionId}/merge`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"requestBody":{"type":"object","required":["source_record_ids","target_record_id"],"properties":{"source_record_ids":{"type":"array","items":{"type":"string"},"description":"Record IDs to merge (first one becomes the target)"},"target_record_id":{"type":"string","description":"Target record ID (must be in source_record_ids)"},"add_fields":{"type":"array","items":{"type":"string"},"description":"Field names to add (numeric values will be summed)"},"delete_sources":{"type":"boolean","description":"Whether to delete source records after merge (default: false)"}},"description":"The JSON request body."}},"required":["collectionId","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}/merge",
    executionParameters: [{"name":"collectionId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_read", {
    name: "RecordApi_read",
    description: `Executes GET /records/{collectionId}/{recordId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"}},"required":["collectionId","recordId"]},
    method: "get",
    pathTemplate: "/records/{collectionId}/{recordId}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["RecordApi_update", {
    name: "RecordApi_update",
    description: `Executes PUT /records/{collectionId}/{recordId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"requestBody":{"type":"object","required":["data"],"properties":{"data":{"type":"object","unevaluatedProperties":{"type":"object"}}},"description":"The JSON request body."}},"required":["collectionId","recordId","requestBody"]},
    method: "put",
    pathTemplate: "/records/{collectionId}/{recordId}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_delete", {
    name: "RecordApi_delete",
    description: `Executes DELETE /records/{collectionId}/{recordId}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"}},"required":["collectionId","recordId"]},
    method: "delete",
    pathTemplate: "/records/{collectionId}/{recordId}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["RecordApi_copyRecord", {
    name: "RecordApi_copyRecord",
    description: `Executes POST /records/{collectionId}/{recordId}/copy`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"requestBody":{"type":"object","required":["target_collection_id"],"properties":{"target_collection_id":{"type":"string","description":"Target collection ID to copy the record to"}},"description":"The JSON request body."}},"required":["collectionId","recordId","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}/{recordId}/copy",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_getImage", {
    name: "RecordApi_getImage",
    description: `Executes GET /records/{collectionId}/{recordId}/images/{fieldName}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"fieldName":{"type":"string"}},"required":["collectionId","recordId","fieldName"]},
    method: "get",
    pathTemplate: "/records/{collectionId}/{recordId}/images/{fieldName}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"},{"name":"fieldName","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["RecordApi_uploadImage", {
    name: "RecordApi_uploadImage",
    description: `Executes POST /records/{collectionId}/{recordId}/images/{fieldName}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"fieldName":{"type":"string"},"requestBody":{"type":"object","required":["data","content_type"],"properties":{"data":{"type":"string","description":"Base64 encoded image data"},"content_type":{"type":"string","enum":["image/png","image/jpeg","image/webp","image/gif"],"description":"MIME type"},"filename":{"type":"string","description":"Optional filename"}},"description":"The JSON request body."}},"required":["collectionId","recordId","fieldName","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}/{recordId}/images/{fieldName}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"},{"name":"fieldName","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["RecordApi_deleteImage", {
    name: "RecordApi_deleteImage",
    description: `Executes DELETE /records/{collectionId}/{recordId}/images/{fieldName}`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"fieldName":{"type":"string"}},"required":["collectionId","recordId","fieldName"]},
    method: "delete",
    pathTemplate: "/records/{collectionId}/{recordId}/images/{fieldName}",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"},{"name":"fieldName","in":"path"}],
    requestBodyContentType: undefined,
    securityRequirements: []
  }],
  ["RecordApi_splitRecord", {
    name: "RecordApi_splitRecord",
    description: `Executes POST /records/{collectionId}/{recordId}/split`,
    inputSchema: {"type":"object","properties":{"collectionId":{"type":"string"},"recordId":{"type":"string"},"requestBody":{"type":"object","required":["splits"],"properties":{"splits":{"type":"array","items":{"type":"object","unevaluatedProperties":{"type":"object"}},"description":"Split specifications: each object contains field names and values to subtract"}},"description":"The JSON request body."}},"required":["collectionId","recordId","requestBody"]},
    method: "post",
    pathTemplate: "/records/{collectionId}/{recordId}/split",
    executionParameters: [{"name":"collectionId","in":"path"},{"name":"recordId","in":"path"}],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
  ["ScriptApi_execute", {
    name: "ScriptApi_execute",
    description: `Executes POST /scripts`,
    inputSchema: {"type":"object","properties":{"requestBody":{"type":"object","required":["language","code"],"properties":{"language":{"type":"string","enum":["python3","nodejs"]},"code":{"type":"string"},"preload":{"type":"string"},"enable_network":{"type":"boolean"}},"description":"The JSON request body."}},"required":["requestBody"]},
    method: "post",
    pathTemplate: "/scripts",
    executionParameters: [],
    requestBodyContentType: "application/json",
    securityRequirements: []
  }],
]);

/**
 * Security schemes from the OpenAPI spec
 */
const securitySchemes =   {};


server.setRequestHandler(ListToolsRequestSchema, async () => {
  const toolsForClient: Tool[] = Array.from(toolDefinitionMap.values()).map(def => ({
    name: def.name,
    description: def.description,
    inputSchema: def.inputSchema
  }));
  return { tools: toolsForClient };
});


server.setRequestHandler(CallToolRequestSchema, async (request: CallToolRequest): Promise<CallToolResult> => {
  const { name: toolName, arguments: toolArgs } = request.params;
  const toolDefinition = toolDefinitionMap.get(toolName);
  if (!toolDefinition) {
    console.error(`Error: Unknown tool requested: ${toolName}`);
    return { content: [{ type: "text", text: `Error: Unknown tool requested: ${toolName}` }] };
  }
  return await executeApiTool(toolName, toolDefinition, toolArgs ?? {}, securitySchemes);
});



/**
 * Type definition for cached OAuth tokens
 */
interface TokenCacheEntry {
    token: string;
    expiresAt: number;
}

/**
 * Declare global __oauthTokenCache property for TypeScript
 */
declare global {
    var __oauthTokenCache: Record<string, TokenCacheEntry> | undefined;
}

/**
 * Acquires an OAuth2 token using client credentials flow
 * 
 * @param schemeName Name of the security scheme
 * @param scheme OAuth2 security scheme
 * @returns Acquired token or null if unable to acquire
 */
async function acquireOAuth2Token(schemeName: string, scheme: any): Promise<string | null | undefined> {
    try {
        // Check if we have the necessary credentials
        const clientId = process.env[`OAUTH_CLIENT_ID_SCHEMENAME`];
        const clientSecret = process.env[`OAUTH_CLIENT_SECRET_SCHEMENAME`];
        const scopes = process.env[`OAUTH_SCOPES_SCHEMENAME`];
        
        if (!clientId || !clientSecret) {
            console.error(`Missing client credentials for OAuth2 scheme '${schemeName}'`);
            return null;
        }
        
        // Initialize token cache if needed
        if (typeof global.__oauthTokenCache === 'undefined') {
            global.__oauthTokenCache = {};
        }
        
        // Check if we have a cached token
        const cacheKey = `${schemeName}_${clientId}`;
        const cachedToken = global.__oauthTokenCache[cacheKey];
        const now = Date.now();
        
        if (cachedToken && cachedToken.expiresAt > now) {
            console.error(`Using cached OAuth2 token for '${schemeName}' (expires in ${Math.floor((cachedToken.expiresAt - now) / 1000)} seconds)`);
            return cachedToken.token;
        }
        
        // Determine token URL based on flow type
        let tokenUrl = '';
        if (scheme.flows?.clientCredentials?.tokenUrl) {
            tokenUrl = scheme.flows.clientCredentials.tokenUrl;
            console.error(`Using client credentials flow for '${schemeName}'`);
        } else if (scheme.flows?.password?.tokenUrl) {
            tokenUrl = scheme.flows.password.tokenUrl;
            console.error(`Using password flow for '${schemeName}'`);
        } else {
            console.error(`No supported OAuth2 flow found for '${schemeName}'`);
            return null;
        }
        
        // Prepare the token request
        let formData = new URLSearchParams();
        formData.append('grant_type', 'client_credentials');
        
        // Add scopes if specified
        if (scopes) {
            formData.append('scope', scopes);
        }
        
        console.error(`Requesting OAuth2 token from ${tokenUrl}`);
        
        // Make the token request
        const response = await axios({
            method: 'POST',
            url: tokenUrl,
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                'Authorization': `Basic ${Buffer.from(`${clientId}:${clientSecret}`).toString('base64')}`
            },
            data: formData.toString()
        });
        
        // Process the response
        if (response.data?.access_token) {
            const token = response.data.access_token;
            const expiresIn = response.data.expires_in || 3600; // Default to 1 hour
            
            // Cache the token
            global.__oauthTokenCache[cacheKey] = {
                token,
                expiresAt: now + (expiresIn * 1000) - 60000 // Expire 1 minute early
            };
            
            console.error(`Successfully acquired OAuth2 token for '${schemeName}' (expires in ${expiresIn} seconds)`);
            return token;
        } else {
            console.error(`Failed to acquire OAuth2 token for '${schemeName}': No access_token in response`);
            return null;
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        console.error(`Error acquiring OAuth2 token for '${schemeName}':`, errorMessage);
        return null;
    }
}


/**
 * Executes an API tool with the provided arguments
 * 
 * @param toolName Name of the tool to execute
 * @param definition Tool definition
 * @param toolArgs Arguments provided by the user
 * @param allSecuritySchemes Security schemes from the OpenAPI spec
 * @returns Call tool result
 */
async function executeApiTool(
    toolName: string,
    definition: McpToolDefinition,
    toolArgs: JsonObject,
    allSecuritySchemes: Record<string, any>
): Promise<CallToolResult> {
  try {
    // Validate arguments against the input schema
    let validatedArgs: JsonObject;
    try {
        const zodSchema = getZodSchemaFromJsonSchema(definition.inputSchema, toolName);
        const argsToParse = (typeof toolArgs === 'object' && toolArgs !== null) ? toolArgs : {};
        validatedArgs = zodSchema.parse(argsToParse);
    } catch (error: unknown) {
        if (error instanceof ZodError) {
            const validationErrorMessage = `Invalid arguments for tool '${toolName}': ${error.errors.map(e => `${e.path.join('.')} (${e.code}): ${e.message}`).join(', ')}`;
            return { content: [{ type: 'text', text: validationErrorMessage }] };
        } else {
             const errorMessage = error instanceof Error ? error.message : String(error);
             return { content: [{ type: 'text', text: `Internal error during validation setup: ${errorMessage}` }] };
        }
    }

    // Prepare URL, query parameters, headers, and request body
    let urlPath = definition.pathTemplate;
    const queryParams: Record<string, any> = {};
    const headers: Record<string, string> = { 'Accept': 'application/json' };
    let requestBodyData: any = undefined;

    // Apply parameters to the URL path, query, or headers
    definition.executionParameters.forEach((param) => {
        const value = validatedArgs[param.name];
        if (typeof value !== 'undefined' && value !== null) {
            if (param.in === 'path') {
                urlPath = urlPath.replace(`{${param.name}}`, encodeURIComponent(String(value)));
            }
            else if (param.in === 'query') {
                queryParams[param.name] = value;
            }
            else if (param.in === 'header') {
                headers[param.name.toLowerCase()] = String(value);
            }
        }
    });

    // Ensure all path parameters are resolved
    if (urlPath.includes('{')) {
        throw new Error(`Failed to resolve path parameters: ${urlPath}`);
    }
    
    // Construct the full URL
    const requestUrl = API_BASE_URL ? `${API_BASE_URL}${urlPath}` : urlPath;

    // Handle request body if needed
    if (definition.requestBodyContentType && typeof validatedArgs['requestBody'] !== 'undefined') {
        requestBodyData = validatedArgs['requestBody'];
        headers['content-type'] = definition.requestBodyContentType;
    }


    // Apply security requirements if available
    // Security requirements use OR between array items and AND within each object
    const appliedSecurity = definition.securityRequirements?.find(req => {
        // Try each security requirement (combined with OR)
        return Object.entries(req).every(([schemeName, scopesArray]) => {
            const scheme = allSecuritySchemes[schemeName];
            if (!scheme) return false;
            
            // API Key security (header, query, cookie)
            if (scheme.type === 'apiKey') {
                return !!process.env[`API_KEY_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
            }
            
            // HTTP security (basic, bearer)
            if (scheme.type === 'http') {
                if (scheme.scheme?.toLowerCase() === 'bearer') {
                    return !!process.env[`BEARER_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                }
                else if (scheme.scheme?.toLowerCase() === 'basic') {
                    return !!process.env[`BASIC_USERNAME_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`] && 
                           !!process.env[`BASIC_PASSWORD_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                }
            }
            
            // OAuth2 security
            if (scheme.type === 'oauth2') {
                // Check for pre-existing token
                if (process.env[`OAUTH_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`]) {
                    return true;
                }
                
                // Check for client credentials for auto-acquisition
                if (process.env[`OAUTH_CLIENT_ID_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`] &&
                    process.env[`OAUTH_CLIENT_SECRET_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`]) {
                    // Verify we have a supported flow
                    if (scheme.flows?.clientCredentials || scheme.flows?.password) {
                        return true;
                    }
                }
                
                return false;
            }
            
            // OpenID Connect
            if (scheme.type === 'openIdConnect') {
                return !!process.env[`OPENID_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
            }
            
            return false;
        });
    });

    // If we found matching security scheme(s), apply them
    if (appliedSecurity) {
        // Apply each security scheme from this requirement (combined with AND)
        for (const [schemeName, scopesArray] of Object.entries(appliedSecurity)) {
            const scheme = allSecuritySchemes[schemeName];
            
            // API Key security
            if (scheme?.type === 'apiKey') {
                const apiKey = process.env[`API_KEY_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                if (apiKey) {
                    if (scheme.in === 'header') {
                        headers[scheme.name.toLowerCase()] = apiKey;
                        console.error(`Applied API key '${schemeName}' in header '${scheme.name}'`);
                    }
                    else if (scheme.in === 'query') {
                        queryParams[scheme.name] = apiKey;
                        console.error(`Applied API key '${schemeName}' in query parameter '${scheme.name}'`);
                    }
                    else if (scheme.in === 'cookie') {
                        // Add the cookie, preserving other cookies if they exist
                        headers['cookie'] = `${scheme.name}=${apiKey}${headers['cookie'] ? `; ${headers['cookie']}` : ''}`;
                        console.error(`Applied API key '${schemeName}' in cookie '${scheme.name}'`);
                    }
                }
            } 
            // HTTP security (Bearer or Basic)
            else if (scheme?.type === 'http') {
                if (scheme.scheme?.toLowerCase() === 'bearer') {
                    const token = process.env[`BEARER_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                    if (token) {
                        headers['authorization'] = `Bearer ${token}`;
                        console.error(`Applied Bearer token for '${schemeName}'`);
                    }
                } 
                else if (scheme.scheme?.toLowerCase() === 'basic') {
                    const username = process.env[`BASIC_USERNAME_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                    const password = process.env[`BASIC_PASSWORD_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                    if (username && password) {
                        headers['authorization'] = `Basic ${Buffer.from(`${username}:${password}`).toString('base64')}`;
                        console.error(`Applied Basic authentication for '${schemeName}'`);
                    }
                }
            }
            // OAuth2 security
            else if (scheme?.type === 'oauth2') {
                // First try to use a pre-provided token
                let token = process.env[`OAUTH_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                
                // If no token but we have client credentials, try to acquire a token
                if (!token && (scheme.flows?.clientCredentials || scheme.flows?.password)) {
                    console.error(`Attempting to acquire OAuth token for '${schemeName}'`);
                    token = (await acquireOAuth2Token(schemeName, scheme)) ?? '';
                }
                
                // Apply token if available
                if (token) {
                    headers['authorization'] = `Bearer ${token}`;
                    console.error(`Applied OAuth2 token for '${schemeName}'`);
                    
                    // List the scopes that were requested, if any
                    const scopes = scopesArray as string[];
                    if (scopes && scopes.length > 0) {
                        console.error(`Requested scopes: ${scopes.join(', ')}`);
                    }
                }
            }
            // OpenID Connect
            else if (scheme?.type === 'openIdConnect') {
                const token = process.env[`OPENID_TOKEN_${schemeName.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase()}`];
                if (token) {
                    headers['authorization'] = `Bearer ${token}`;
                    console.error(`Applied OpenID Connect token for '${schemeName}'`);
                    
                    // List the scopes that were requested, if any
                    const scopes = scopesArray as string[];
                    if (scopes && scopes.length > 0) {
                        console.error(`Requested scopes: ${scopes.join(', ')}`);
                    }
                }
            }
        }
    } 
    // Log warning if security is required but not available
    else if (definition.securityRequirements?.length > 0) {
        // First generate a more readable representation of the security requirements
        const securityRequirementsString = definition.securityRequirements
            .map(req => {
                const parts = Object.entries(req)
                    .map(([name, scopesArray]) => {
                        const scopes = scopesArray as string[];
                        if (scopes.length === 0) return name;
                        return `${name} (scopes: ${scopes.join(', ')})`;
                    })
                    .join(' AND ');
                return `[${parts}]`;
            })
            .join(' OR ');
            
        console.warn(`Tool '${toolName}' requires security: ${securityRequirementsString}, but no suitable credentials found.`);
    }
    

    // Prepare the axios request configuration
    const config: AxiosRequestConfig = {
      method: definition.method.toUpperCase(), 
      url: requestUrl, 
      params: queryParams, 
      headers: headers,
      ...(requestBodyData !== undefined && { data: requestBodyData }),
    };

    // Log request info to stderr (doesn't affect MCP output)
    console.error(`Executing tool "${toolName}": ${config.method} ${config.url}`);
    
    // Execute the request
    const response = await axios(config);

    // Process and format the response
    let responseText = '';
    const contentType = response.headers['content-type']?.toLowerCase() || '';
    
    // Handle JSON responses
    if (contentType.includes('application/json') && typeof response.data === 'object' && response.data !== null) {
         try { 
             responseText = JSON.stringify(response.data, null, 2); 
         } catch (e) { 
             responseText = "[Stringify Error]"; 
         }
    } 
    // Handle string responses
    else if (typeof response.data === 'string') { 
         responseText = response.data; 
    }
    // Handle other response types
    else if (response.data !== undefined && response.data !== null) { 
         responseText = String(response.data); 
    }
    // Handle empty responses
    else { 
         responseText = `(Status: ${response.status} - No body content)`; 
    }
    
    // Return formatted response
    return { 
        content: [ 
            { 
                type: "text", 
                text: `API Response (Status: ${response.status}):\n${responseText}` 
            } 
        ], 
    };

  } catch (error: unknown) {
    // Handle errors during execution
    let errorMessage: string;
    
    // Format Axios errors specially
    if (axios.isAxiosError(error)) { 
        errorMessage = formatApiError(error); 
    }
    // Handle standard errors
    else if (error instanceof Error) { 
        errorMessage = error.message; 
    }
    // Handle unexpected error types
    else { 
        errorMessage = 'Unexpected error: ' + String(error); 
    }
    
    // Log error to stderr
    console.error(`Error during execution of tool '${toolName}':`, errorMessage);
    
    // Return error message to client
    return { content: [{ type: "text", text: errorMessage }] };
  }
}


/**
 * Main function to start the server
 */
async function main() {
// Set up stdio transport
  try {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error(`${SERVER_NAME} MCP Server (v${SERVER_VERSION}) running on stdio${API_BASE_URL ? `, proxying API at ${API_BASE_URL}` : ''}`);
  } catch (error) {
    console.error("Error during server startup:", error);
    process.exit(1);
  }
}

/**
 * Cleanup function for graceful shutdown
 */
async function cleanup() {
    console.error("Shutting down MCP server...");
    process.exit(0);
}

// Register signal handlers
process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

// Start the server
main().catch((error) => {
  console.error("Fatal error in main execution:", error);
  process.exit(1);
});

/**
 * Formats API errors for better readability
 * 
 * @param error Axios error
 * @returns Formatted error message
 */
function formatApiError(error: AxiosError): string {
    let message = 'API request failed.';
    if (error.response) {
        message = `API Error: Status ${error.response.status} (${error.response.statusText || 'Status text not available'}). `;
        const responseData = error.response.data;
        const MAX_LEN = 200;
        if (typeof responseData === 'string') { 
            message += `Response: ${responseData.substring(0, MAX_LEN)}${responseData.length > MAX_LEN ? '...' : ''}`; 
        }
        else if (responseData) { 
            try { 
                const jsonString = JSON.stringify(responseData); 
                message += `Response: ${jsonString.substring(0, MAX_LEN)}${jsonString.length > MAX_LEN ? '...' : ''}`; 
            } catch { 
                message += 'Response: [Could not serialize data]'; 
            } 
        }
        else { 
            message += 'No response body received.'; 
        }
    } else if (error.request) {
        message = 'API Network Error: No response received from server.';
        if (error.code) message += ` (Code: ${error.code})`;
    } else { 
        message += `API Request Setup Error: ${error.message}`; 
    }
    return message;
}

/**
 * Converts a JSON Schema to a Zod schema for runtime validation
 * 
 * @param jsonSchema JSON Schema
 * @param toolName Tool name for error reporting
 * @returns Zod schema
 */
function getZodSchemaFromJsonSchema(jsonSchema: any, toolName: string): z.ZodTypeAny {
    if (typeof jsonSchema !== 'object' || jsonSchema === null) { 
        return z.object({}).passthrough(); 
    }
    try {
        const zodSchemaString = jsonSchemaToZod(jsonSchema);
        const zodSchema = eval(zodSchemaString);
        if (typeof zodSchema?.parse !== 'function') { 
            throw new Error('Eval did not produce a valid Zod schema.'); 
        }
        return zodSchema as z.ZodTypeAny;
    } catch (err: any) {
        console.error(`Failed to generate/evaluate Zod schema for '${toolName}':`, err);
        return z.object({}).passthrough();
    }
}
