<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/kodeflowstudios/utui">
    <img src="images/logo.jpg" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">UTUI</h3>

  <p align="center">
    An awesome TUI interface for Unity's Unity CLI command line tool.
    <br />
    <br />
⚠️ This project is an independent, open-source project and is not affiliated with or endorsed by Unity Software Inc.
    <br />
    <br />
    <a href="https://kodeflowstudios.com">KodeFlow Studios</a>
    &middot;
    <a href="https://github.com/kodeflowstudios/utui/issues/new?labels=bug&template=bug-report.md">Report Bug</a>
    &middot;
    <a href="https://github.com/kodeflowstudios/utui/issues/new?labels=improvement&template=feature-request.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

[![Product Name Screen Shot][product-screenshot]](https://kodeflowstudios.com)

There are times where you want to stay inside of your terminal to view/manage your unity project, and with the addition of Unity's new CLI tool you can do that but at the cost of slowly typing away lengthy commands with paths and editor versions that are quite verbose—but with UTUI you can do all of that directly from a beautiful TUI interface.

Here's why you should use it:
- Much faster workflow
- Create/Delete/Open projects directly form the interface
- Install/Uninstall editors
- Add and manage commands (coming soon)
- Create command templates (coming soon)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

   [![Rust][Rust]][Rust-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started
### Prerequisites

* [rustup](https://rustup.rs/) — only required if building from source
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Installation

#### macOS

**Apple Silicon (M1/M2/M3/M4)**

Using Homebrew:

```sh
brew install kodeflowstudios/tap/utui
```

Or download the latest release manually:

```sh
curl -LO https://github.com/kodeflowstudios/utui/releases/latest/download/utui-aarch64-apple-darwin.tar.gz
tar -xzf utui-aarch64-apple-darwin.tar.gz
sudo mv utui /usr/local/bin/utui
```

**Intel (x86_64)**

Using Homebrew:

```sh
brew install kodeflowstudios/tap/utui
```

Or download the latest release manually:

```sh
curl -LO https://github.com/kodeflowstudios/utui/releases/latest/download/utui-x86_64-apple-darwin.tar.gz
tar -xzf utui-x86_64-apple-darwin.tar.gz
sudo mv utui /usr/local/bin/utui
```

#### Linux

**x86_64 (Intel/AMD)**

```sh
curl -LO https://github.com/kodeflowstudios/utui/releases/latest/download/utui-x86_64-unknown-linux-gnu.tar.gz
tar -xzf utui-x86_64-unknown-linux-gnu.tar.gz
sudo mv utui /usr/local/bin/utui
```

### Verify installation

```sh
utui --version
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

```sh
utui
```

Refere to the help menu inside of UTUI for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] Custom commands
    - [ ] Viewing custom commands
    - [ ] Executing custom commands
    - [ ] Creating custom commands
    - [ ] Custom command templates and opening C# file an editor

See the [open issues](https://github.com/kodeflowstudios/utui/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".

Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/kodeflowstudios/utui/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=kodeflowstudios/utui" alt="Contributor preview unavailable..." />
</a>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Your Name - [@kodeflowstudios](https://x.com/kodeflowstudios) - contact@kodeflowstudios.com

Project Link: [https://github.com/kodeflowstudios/utui](https://github.com/github_username/repo_name)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [Rust Docs](https://doc.rust-lang.org/book/)
* [Ratatui Docs](https://ratatui.rs/tutorials/)
* [UnityCLI Docs](https://docs.unity.com/en-us/unity-cli)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/kodeflowstudios/utui.svg?style=for-the-badge
[contributors-url]: https://github.com/kodeflowstudios/utui/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/kodeflowstudios/utui.svg?style=for-the-badge
[forks-url]: https://github.com/kodeflowstudios/utui/network/members
[stars-shield]: https://img.shields.io/github/stars/kodeflowstudios/utui.svg?style=for-the-badge
[stars-url]: https://github.com/kodeflowstudios/utui/stargazers
[issues-shield]: https://img.shields.io/github/issues/kodeflowstudios/utui.svg?style=for-the-badge
[issues-url]: https://github.com/kodeflowstudios/utui/issues
[license-shield]: https://img.shields.io/github/license/kodeflowstudios/utui.svg?style=for-the-badge
[license-url]: https://github.com/kodeflowstudios/utui/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: www.linkedin.com/in/ahmed-elkadii
[product-screenshot]: images/screenshot.png
<!-- Shields.io badges. You can a comprehensive list with many more badges at: https://github.com/inttter/md-badges -->
[Rust]: https://img.shields.io/badge/rust-F26F18?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://rust-lang.org/
[Next.js]: https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white
[Next-url]: https://nextjs.org/
[React.js]: https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Vue.js]: https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D
[Vue-url]: https://vuejs.org/
[Angular.io]: https://img.shields.io/badge/Angular-DD0031?style=for-the-badge&logo=angular&logoColor=white
[Angular-url]: https://angular.io/
[Svelte.dev]: https://img.shields.io/badge/Svelte-4A4A55?style=for-the-badge&logo=svelte&logoColor=FF3E00
[Svelte-url]: https://svelte.dev/
[Laravel.com]: https://img.shields.io/badge/Laravel-FF2D20?style=for-the-badge&logo=laravel&logoColor=white
[Laravel-url]: https://laravel.com
[Bootstrap.com]: https://img.shields.io/badge/Bootstrap-563D7C?style=for-the-badge&logo=bootstrap&logoColor=white
[Bootstrap-url]: https://getbootstrap.com
[JQuery.com]: https://img.shields.io/badge/jQuery-0769AD?style=for-the-badge&logo=jquery&logoColor=white
[JQuery-url]: https://jquery.com 
