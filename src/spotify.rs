// use url::Url;
struct UrlString(String);

// impl TryInto<rspotify::model::AlbumId<'_>> for UrlString {
//     type Error = anyhow::Error;
//
//     fn try_into(self) -> anyhow::Result<rspotify::model::AlbumId<'static>, Self::Error> {
//         let url = Url::parse(self.0.as_ref())?;
//         let seg = url
//             .path_segments()
//             .ok_or(anyhow::anyhow!("no path segments"))?;
//
//         let id = seg.last().ok_or(anyhow::anyhow!("no segments"))?;
//         let uri = format!("spotify:album:{id}");
//         let album_id =
//             rspotify::model::AlbumId::from_uri(["spotify", "album", &id].join(":").as_ref())?;
//         Ok(album_id)
//     }
// }
