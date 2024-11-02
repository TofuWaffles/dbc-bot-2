import { Pool } from 'pg';
import * as dotenv from "dotenv";
import { Err, expect, Ok } from '@/utils';
dotenv.config({ path: `${process.cwd()}/../.env` });

export class EnvVars {
    private static instance: EnvVars | null = null;

    private constructor(private DATABASE_URL: string, private BRAWL_STARS_TOKEN?: string) {}

    public static fromEnv(): Result<EnvVars> {
        if (!process.env.DATABASE_URL) {
            return Err(new Error("Required environment variables are missing."));
        }
        if (!this.instance) {
            this.instance = new EnvVars(process.env.DATABASE_URL, process.env.BRAWL_STARS_TOKEN);
        }
        return Ok(this.instance);
    }

    public getDatabaseUrl(): string {
        return this.DATABASE_URL;
    }

    public getBrawlStarsToken(): string | undefined {
        return this.BRAWL_STARS_TOKEN;
    }
}


const envVars = expect(EnvVars.fromEnv());

export const pool: Pool = new Pool({
    connectionString: envVars.getDatabaseUrl()
});

export const BSToken: string = envVars.getBrawlStarsToken();