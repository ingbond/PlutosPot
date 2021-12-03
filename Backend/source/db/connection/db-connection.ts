// import {Error, PoolOptions, Query, SqlClient, QueryDescription, Pool, PoolStatusRecord} from 'msnodesqlv8';
// export const sql: SqlClient = require('msnodesqlv8');

// export let mainpool: Pool;

// export function initConnection() {
//   const pool = new sql.Pool({
//     connectionString: process.env.CONNECTION_STRING!
//   });
//   pool.open((e: Error, options:PoolOptions) => {
//     if (e) {
//         console.log(`Error ${e.message}`)
//     } else {
//         console.log(JSON.stringify(options,null, 4))
//     }
//   });
//   mainpool = pool;
// }

/*
   server: process.env.SQL_SERVER!,
    database: process.env.SQL_DATABASE!, */