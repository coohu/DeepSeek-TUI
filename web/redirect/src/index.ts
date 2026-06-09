export default {
  fetch(request: Request): Response {
    const url = new URL(request.url);
    url.host = "deepseek.net";
    return Response.redirect(url.toString(), 301);
  },
};
