import { cloneElement, useCallback, useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import Modal from "react-modal";
import { useDispatch, useSelector } from "react-redux";

import { addNotification } from "../../slices/notifications";
import MediaTypeSelection from "./MediaTypeSelection";
import Button from "../../Components/Misc/Button";

import Search from "./Search";
import { RematchContext } from "./Context";

import "./Index.scss";

Modal.setAppElement("body");

function RematchMediaModal(props) {
  const dispatch = useDispatch();

  const { id } = useParams();

  const { token } = useSelector((store) => ({
    token: store.auth.token,
  }));

  const [visible, setVisible] = useState(false);

  const [query, setQuery] = useState("");
  const [tmdbResults, setTmdbResults] = useState([]);
  const [mediaType, setMediaType] = useState("movie");
  const [tmdbID, setTmdbID] = useState();
  const [externalID, setExternalID] = useState("");
  const [error, setError] = useState("");
  const [matching, setMatching] = useState(false);

  const clearData = useCallback(() => {
    setTmdbID();
    setExternalID("");
    setTmdbResults([]);
    setError();
    setQuery();
    setMatching(false);
  }, []);

  // prevent scrolling behind Modal
  useEffect(() => {
    visible
      ? (document.body.style.overflow = "hidden")
      : (document.body.style.overflow = "unset");
  }, [visible]);

  const close = useCallback(() => {
    setVisible(false);
    clearData();

    if (props.cleanUp) {
      props.cleanUp();
    }
  }, [props, clearData]);

  const open = useCallback(() => {
    setVisible(true);
  }, []);

  const rematch = useCallback(async () => {
    // Acepta un id de TMDB (numerico) o de IMDb (tt...) escrito a mano,
    // o el resultado seleccionado en la busqueda.
    const targetID = externalID.trim() || tmdbID;

    if (!targetID || !mediaType) return;

    setMatching(true);
    setError("");

    const config = {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        authorization: token,
      },
      body: JSON.stringify({
        external_id: String(targetID),
        media_type: mediaType,
      }),
    };

    const req = await fetch(`/api/v1/media/${id}/rematch`, config);

    if (req.status !== 200) {
      const message = await req.text().catch(() => req.statusText);

      setError(message || req.statusText);
      setMatching(false);

      return;
    }

    dispatch(
      addNotification({
        msg: `Sucessfuly matched ${id}.`,
      })
    );

    clearData();
    close();
  }, [
    clearData,
    dispatch,
    mediaType,
    id,
    setError,
    setMatching,
    tmdbID,
    externalID,
    token,
    close,
  ]);

  const initialValue = {
    mediaType,
    setMediaType,
    tmdbResults,
    setTmdbResults,
    query,
    setQuery,
    tmdbID,
    setTmdbID,
  };

  const hasTarget = Boolean(externalID.trim() || tmdbID);

  return (
    <RematchContext.Provider value={initialValue}>
      <div className="modalBoxContainer">
        {cloneElement(props.children, { onClick: () => open() })}
        <Modal
          isOpen={visible}
          className="modalBox"
          id="modalRematchMedia"
          onRequestClose={close}
          overlayClassName="popupOverlay"
        >
          <div className="modalRematchMedia">
            <div className="heading">
              <h3>Rematch</h3>
              <div className="separator" />
            </div>
            <MediaTypeSelection
              mediaType={mediaType}
              setMediaType={setMediaType}
            />
            <div className="idMatch">
              <p className="desc">
                Or match directly with an ID: TMDB (numeric) or IMDb (tt...).
              </p>
              <input
                value={externalID}
                onChange={(e) => setExternalID(e.target.value)}
                placeholder="603692 or tt1160419"
                spellCheck={false}
              />
            </div>
            <div className="search-section">
              <Search />
            </div>
            {error && <p className="err">Error: {error}</p>}
            <div className="options">
              <Button type="secondary" onClick={close}>
                Nevermind
              </Button>
              <Button onClick={rematch} disabled={matching || !hasTarget}>
                Match
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </RematchContext.Provider>
  );
}

export default RematchMediaModal;
