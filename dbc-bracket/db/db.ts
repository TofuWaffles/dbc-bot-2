import { Pool } from 'pg';
import * as dotenv from "dotenv";
dotenv.config({ path: __dirname+'../../../../../../.env' });

export class EnvVars {
    private DATABASE_URL: string;
    private BRAWL_STARS_TOKEN: string;

    public constructor(databaseUrl: string, brawlStarsToken: string) {
        this.DATABASE_URL = databaseUrl;
        this.BRAWL_STARS_TOKEN = brawlStarsToken;
    }

    public static fromEnv(): EnvVars {
        if (!process.env.DATABASE_URL 
            // || !process.env.BRAWL_STARS_TOKEN
        ) {
            throw new Error("Required environment variables are missing.");
        }
        return new EnvVars(
            process.env.DATABASE_URL || '', 
            process.env.BRAWL_STARS_TOKEN || ''
        );
    }

    public getDatabaseUrl(): string {
        return this.DATABASE_URL;
    }

    public getBrawlStarsToken(): string {
        return this.BRAWL_STARS_TOKEN;
    }
}

const envVars = EnvVars.fromEnv();

export const pool: Pool = new Pool({
    connectionString: envVars.getDatabaseUrl()
});

export const BSToken: string = envVars.getBrawlStarsToken();