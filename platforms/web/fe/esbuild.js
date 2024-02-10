const { build } = require("esbuild");

 build({
   entryPoints: ['src/index.js'],
   outdir: 'dist/build',
   bundle: true,
   minify: false,
   define: {
     Buffer: 'Buffer'
   },
}).catch( () => {
  return process.exit(1);
});
