import { Pool } from 'pg';

import { Err, expect, Ok } from '@/utils';
// import * as dotenv from "dotenv";
// dotenv.config({ path: `${process.cwd()}/../.env` });

export class EnvVars {
    private static instance: EnvVars | null = null;

    private constructor(private DATABASE_URL) {}

    public static fromEnv(): Result<EnvVars> {
        if (!process.env.DATABASE_URL) {
            return Err(new Error("Required environment variables are missing."));
        }
        if (!this.instance) {
            this.instance = new EnvVars(process.env.DATABASE_URL);
        }
        return Ok(this.instance);
    }

    public getDatabaseUrl(): string {
        return this.DATABASE_URL;
    }
}


const envVars = expect(EnvVars.fromEnv());

export const pool: Pool = new Pool({
    connectionString: envVars.getDatabaseUrl()
});
