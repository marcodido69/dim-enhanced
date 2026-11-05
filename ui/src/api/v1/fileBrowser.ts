import v1 from "./";

export const fileBrowser = v1.injectEndpoints({
  endpoints: (build) => ({
    getDirectories: build.query<string[], string>({
      query: (path) => {
        // La raiz se pide como `filebrowser` (sin barra final) para que pegue
        // con la ruta del backend y no con el fallback del SPA.
        const clean = (path || "").replace(/^\/+/, "").replace(/\/+$/, "");

        return clean ? `filebrowser/${clean}` : "filebrowser";
      },
    }),
  }),
});

export const { useGetDirectoriesQuery } = fileBrowser;

export default fileBrowser;
