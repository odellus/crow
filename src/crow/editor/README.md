# crow editor

This is hard.


I have a lot of ideas about what we want this to be, so far I have basically a "cognitive nozzle" that takes turbulent nonlinear ideas/workflows and tries to straighten them out.

What it needs to be partly:

1. ACP client - This is really an ACP client. That's going to be the main thing with the editor. Yeah you can edit files and have beautiful WYSIWYG editing of mystmd or some flavor of markdown we can turn into prosemirror or something but yeah honestly zed has a text chat feature where you can chat and modify the whole thing. It's marked up like markdown too. They just haven't worked out tool calling. I think we'll do that. `marimo` showed us this could be done and we're immensely grateful to them for it.
2. note taker - Part obsidian, part notion, I'm going to pull in features from `logseq` and perhaps pane or something to organize notes, ideas, and projects/work. I want to pull all the projects into a single directory of the crow server because like why even worry about like "oh what folder am I running my server on????" like make every project some folder under the crow server. filesystems are fractal!
3. `codeblitz` has so many features from being built on top of vs code's monaco I feel like we'd be foolish to endeavor on building any kind of web development environment from scratch. Instead, we should focus on integrating existing tools and technologies to create a seamless development experience. We can leverage the power of Monaco Editor, which is already integrated into VS Code, to provide a rich and interactive development environment. Additionally, we can explore other tools like Webpack, Babel, and Jest to streamline the development process and ensure code quality.
4. Because of our decision to say "fuck it" and develop web first, we have access to playwright as a built in development tool that can be used to automate testing and debugging of web applications. Playwright supports multiple browsers and platforms, making it a versatile tool for testing and debugging web applications, especially through our MCP server which our agents can access to test their code and the code of their cohorts.
