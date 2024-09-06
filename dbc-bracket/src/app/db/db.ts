import { Pool } from 'pg';
import * as dotenv from "dotenv";
dotenv.config({ path: __dirname+'../../../../../../../.env' });

export const pool: Pool = new Pool({
    connectionString: process.env.DATABASE_URL
});