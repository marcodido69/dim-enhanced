import { useCallback, useEffect } from "react";
import { useDispatch, useSelector } from "react-redux";

import NewLibraryModal from "../../Modals/NewLibrary/Index";
import {
  handleWsNewLibrary,
  handleWsDelLibrary,
  wsScanStart,
  wsScanStop,
} from "../../actions/library.js";
import useWebSocket from "../../hooks/ws";

import Library from "./Library";

function Libraries() {
  const dispatch = useDispatch();

  const user = useSelector((store) => store.user);
  const libraries = useSelector((store) => store.library.fetch_libraries);
  const ws = useWebSocket();

  const handleWS = useCallback(
    async ({ data }) => {
      const payload = JSON.parse(data);

      if (payload.type === "EventStartedScanning") {
        dispatch(wsScanStart(payload.id));
      }

      if (payload.type === "EventStoppedScanning") {
        dispatch(wsScanStop(payload.id));
      }

      if (payload.type === "EventNewLibrary") {
        dispatch(handleWsNewLibrary(payload.id));
      }

      if (payload.type === "EventRemoveLibrary") {
        dispatch(handleWsDelLibrary(payload.id));
      }
    },
    [dispatch]
  );

  useEffect(() => {
    if (!ws) return;

    ws.addEventListener("message", handleWS);
    return () => ws.removeEventListener("message", handleWS);
  }, [handleWS, ws]);

  let libs = [];

  const { fetched, error, items } = libraries;

  // FETCH_LIBRARIES_OK
  if (fetched && !error && items.length > 0) {
    libs = items.map((props) => <Library {...props} key={props.id} />);
  }

  // La seccion se muestra siempre (incluso con 0 bibliotecas) para que el boton
  // "+" quede accesible y se pueda crear la primera biblioteca desde el sidebar.
  return (
    <section className="libraries">
      <header>
        <h4>Libraries</h4>
        {user.info.roles?.includes("owner") && (
          <NewLibraryModal>
            <button className="openNewLibrary" title="Create a new library">
              +
            </button>
          </NewLibraryModal>
        )}
      </header>
      <div className="list">{libs}</div>
      {fetched && items.length === 0 && (
        <p className="noLibraries">No libraries yet</p>
      )}
    </section>
  );
}

export default Libraries;
