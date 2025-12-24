import type { Mcp } from './mcp';

export type BackendStatus = 'connecting' | 'connected' | 'disconnected' | 'failed' | 'restarting';

export interface BackendInfo {
	mcpId: number;
	mcpName: string;
	mcpType: string;
	status: BackendStatus;
	toolCount: number;
	serverInfo: {
		name: string;
		version?: string;
	} | null;
	errorMessage: string | null;
	restartCount: number;
}

export interface GatewayServerConfig {
	enabled: boolean;
	port: number;
	autoStart: boolean;
}

export interface GatewayServerStatus {
	isRunning: boolean;
	port: number;
	url: string;
	mcpEndpoint: string;
	connectedBackends: BackendInfo[];
	totalTools: number;
}

export interface GatewayMcp {
	id: number;
	mcpId: number;
	mcp: Mcp;
	isEnabled: boolean;
	autoRestart: boolean;
	displayOrder: number;
	createdAt: string;
}
