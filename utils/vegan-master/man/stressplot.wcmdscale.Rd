\name{stressplot.wcmdscale}
\alias{stressplot.wcmdscale}
\alias{stressplot.cca}
\alias{stressplot.rda}
\alias{stressplot.capscale}
\alias{stressplot.dbrda}
\alias{stressplot.prcomp}
\alias{stressplot.princomp}

\title{
  Display Ordination Distances Against Observed Distances in Eigenvector Ordinations
}

\description{
  Functions plot ordination distances in given number of dimensions
  against observed distances or distances in full space in eigenvector
  methods. The display is similar as the Shepard diagram
  (\code{\link{stressplot}} for non-metric multidimensional scaling
  with \code{\link{metaMDS}} or \code{\link{monoMDS}}), but shows the
  linear relationship of the eigenvector ordinations. The
  \code{stressplot} methods are available for \code{\link{wcmdscale}},
  \code{\link{rda}}, \code{\link{cca}}, \code{\link{capscale}},
  \code{\link{dbrda}}, \code{\link{prcomp}} and \code{\link{princomp}}. 
}

\usage{
\method{stressplot}{wcmdscale}(object, k = 2, pch, p.col = "blue", l.col = "red",
    lwd = 2, ...)
}

\arguments{
  \item{object}{
    Result object from eigenvector ordination (\code{\link{wcmdscale}},
    \code{\link{rda}}, \code{\link{cca}}, \code{\link{dbrda}},
    \code{\link{capscale}})
}
  \item{k}{
    Number of dimensions for which the ordination distances are displayed.
}
  \item{pch, p.col, l.col, lwd}{
    Plotting character, point colour and line colour like in
    default \code{\link{stressplot}}
}
  \item{\dots}{
    Other parameters to functions, e.g. graphical parameters.
}
}

\details{ The functions offer a similar display for eigenvector
  ordinations as the standard Shepard diagram
  (\code{\link{stressplot}}) in non-metric multidimensional
  scaling. The ordination distances in given number of dimensions are
  plotted against observed distances. With metric distances, the
  ordination distances in full space (with all ordination axes) are
  equal to observed distances, and the fit line shows this
  equality. In general, the fit line does not go through the points,
  but the points for observed distances approach the fit line from
  below. However, with non-Euclidean distances (in
  \code{\link{wcmdscale}}, \code{\link{dbrda}} or
  \code{\link{capscale}}) with negative eigenvalues the ordination
  distances can exceed the observed distances in real dimensions; the
  imaginary dimensions with negative eigenvalues will correct these
  excess distances. If you have used \code{\link{dbrda}},
  \code{\link{capscale}} or \code{\link{wcmdscale}} with argument
  \code{add} to avoid negative eigenvalues, the ordination distances
  will exceed the observed dissimilarities.

  In partial ordination (\code{\link{cca}}, \code{\link{rda}},
  \code{\link{dbrda}} and \code{\link{capscale}} with \code{Condition}
  in the formula), the distances in the partial component are included
  both in the observed distances and in ordination distances.  With
  \code{k=0}, the ordination distances refer to the partial
  ordination.

}

\value{
  Functions draw a graph and return invisibly the ordination distances
  or the ordination distances.
}

\author{
  Jari Oksanen.
}

\seealso{
  \code{\link{stressplot}} and \code{\link{stressplot.monoMDS}} for
  standard Shepard diagrams.
}

\examples{
data(dune, dune.env)
mod <- rda(dune)
stressplot(mod)
mod <- rda(dune ~ Management, dune.env)
stressplot(mod, k=3)
}

\keyword{ multivariate }

